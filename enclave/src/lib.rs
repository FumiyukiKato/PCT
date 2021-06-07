// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License..

#![crate_name = "pctenclave"]
#![crate_type = "staticlib"]

#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

extern crate sgx_types;
extern crate sgx_trts;
#[cfg(not(target_env = "sgx"))]
#[macro_use]
extern crate sgx_tstd as std;
extern crate sgx_tcrypto;
extern crate bincode;
extern crate succinct_trie;

use sgx_types::*;
use sgx_tcrypto::*;
use std::vec::Vec;
use std::cell::RefCell;
use std::slice;
use std::boxed::Box;
use std::time::{Instant};
use std::untrusted::time::InstantEx;
use std::sync::atomic::{AtomicPtr, Ordering};

mod constant;
mod primitive;
mod utils;
mod encoded_query_rep;
mod period;
mod query_result;
mod encoded_query_buffer;
mod encoded_result_buffer;
mod encoded_dictionary_buffer;
mod encoded_hash_table;
mod fast_succinct_trie;


use constant::*;
use encoded_query_buffer::EncodedQueryBuffer;
use encoded_result_buffer::EncodedResultBuffer;
use encoded_dictionary_buffer::EncodedDictionaryBuffer;


/* 
SGXのステート
    ステートは全部グローバル変数に持ってヒープにメモリを確保する
*/

pub static ENCODED_QUERY_BUFFER: AtomicPtr<()> = AtomicPtr::new(0 as * mut ());
pub fn get_ref_encoded_query_buffer() -> Option<&'static RefCell<EncodedQueryBuffer>> {
    let ptr = ENCODED_QUERY_BUFFER.load(Ordering::SeqCst) as * mut RefCell<EncodedQueryBuffer>;
    if ptr.is_null() {
        None
    } else {
        Some(unsafe { &* ptr })
    }
}

pub static ENCODED_RESULT_BUFFER: AtomicPtr<()> = AtomicPtr::new(0 as * mut ());
pub fn get_ref_encoded_result_buffer() -> Option<&'static RefCell<EncodedResultBuffer>> {
    let ptr = ENCODED_RESULT_BUFFER.load(Ordering::SeqCst) as * mut RefCell<EncodedResultBuffer>;
    if ptr.is_null() {
        None
    } else {
        Some(unsafe { &* ptr })
    }
}

#[no_mangle]
pub extern "C" fn upload_encoded_query_data(
    total_query_data: *const u8,
    total_size       : usize,
    client_size     : usize,
    query_id_list   : *const u64,
) -> sgx_status_t {
    let whole_start = Instant::now();
    let start = Instant::now();
    _init_encoded_buffers();
    let end = start.elapsed();
    println!("[SGX CLOCK] {}:  {}.{:06} seconds", "buffers initialize", end.as_secs(), end.subsec_nanos() / 1_000);

    let start = Instant::now();
    let total_query_data_vec: Vec<u8> = unsafe {
        slice::from_raw_parts(total_query_data, total_size)
    }.to_vec();
    if total_query_data_vec.len() != total_size {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }

    let query_id_list_vec: Vec<u64> = unsafe {
        slice::from_raw_parts(query_id_list, client_size)
    }.to_vec();
    if query_id_list_vec.len() != client_size {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }
    let end = start.elapsed();
    println!("[SGX CLOCK] {}:  {}.{:06} seconds", "reading", end.as_secs(), end.subsec_nanos() / 1_000);

    /* decryption */
    let start = Instant::now();

    let mut decrypted_query_data_vec: Vec<u8> = vec![1; total_query_data_vec.len()];
    for (i, query_id) in query_id_list_vec.iter().enumerate() {
        let mut counter_block: [u8; 16] = COUNTER_BLOCK;
        let ctr_inc_bits: u32 = SGXSSL_CTR_BITS;

        // Originally shared_key is derived by following Remote Attestation protocol.
        // This is mock of shared key-based encryption.
        let mut shared_key: [u8; 16] = [0; 16];
        shared_key[..8].copy_from_slice(&query_id.to_be_bytes());
        let current_cursor = i*QUERY_BYTES;
        let ret = rsgx_aes_ctr_decrypt(
            &shared_key,
            &total_query_data_vec[current_cursor..current_cursor+QUERY_BYTES],
            &mut counter_block,
            ctr_inc_bits,
            &mut decrypted_query_data_vec[current_cursor..current_cursor+QUERY_BYTES]
        );
        match ret { Ok(()) => {}, Err(_) => { return sgx_status_t::SGX_ERROR_UNEXPECTED; } }    
    }
    let end = start.elapsed();
    println!("[SGX CLOCK] {}:  {}.{:06} seconds", "decrypt each queries", end.as_secs(), end.subsec_nanos() / 1_000);


    /* for more optiizaton this part can be conducted in decryption phase together, but to measure each part */
    let start = Instant::now();
    let mut query_buffer = get_ref_encoded_query_buffer().unwrap().borrow_mut();
    query_buffer.build_query_buffer(decrypted_query_data_vec, query_id_list_vec);
    let end = start.elapsed();
    println!("[SGX CLOCK] {}:  {}.{:06} seconds", "store queies", end.as_secs(), end.subsec_nanos() / 1_000);

    let end = whole_start.elapsed();
    println!("[SGX CLOCK] {}:  {}.{:06} seconds", "whole", end.as_secs(), end.subsec_nanos() / 1_000);
    
    sgx_status_t::SGX_SUCCESS
}

fn _init_encoded_buffers() {

    // initialize query buffer
    let query_buffer = EncodedQueryBuffer::new();
    let query_buffer_box = Box::new(RefCell::<EncodedQueryBuffer>::new(query_buffer));
    let query_buffer_ptr = Box::into_raw(query_buffer_box);
    ENCODED_QUERY_BUFFER.store(query_buffer_ptr as *mut (), Ordering::SeqCst);

    // initialize result buffer
    let result_buffer = EncodedResultBuffer::new();
    let result_buffer_box = Box::new(RefCell::<EncodedResultBuffer>::new(result_buffer));
    let result_buffer_ptr = Box::into_raw(result_buffer_box);
    ENCODED_RESULT_BUFFER.store(result_buffer_ptr as *mut (), Ordering::SeqCst);
}

/*
    Private set intersection
*/
#[no_mangle]
pub extern "C" fn private_encode_contact_trace(
    encoded_value_u8: *const u8,
    encoded_value_u8_size: usize,
) -> sgx_status_t {
    let encoded_value_vec: Vec<u8> = unsafe {
        slice::from_raw_parts(encoded_value_u8, encoded_value_u8_size)
    }.to_vec();
    if encoded_value_vec.len() != encoded_value_u8_size {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }

    /* decryption */
    let start = Instant::now();
    let mut decrypted: Vec<u8> = vec![1; encoded_value_vec.len()];
    let mut counter_block: [u8; 16] = COUNTER_BLOCK;
    let ctr_inc_bits: u32 = SGXSSL_CTR_BITS;
    // Originally shared_key is derived by following Remote Attestation protocol.
    // This is mock of shared key-based encryption.
    let mut shared_key: [u8; 16] = [0; 16];
    shared_key[..8].copy_from_slice(&CENTRAL_KEY.to_be_bytes());
    let ret = rsgx_aes_ctr_decrypt(
        &shared_key,
        &encoded_value_vec,
        &mut counter_block,
        ctr_inc_bits,
        &mut decrypted
    );
    match ret { Ok(()) => {}, Err(_) => { return sgx_status_t::SGX_ERROR_UNEXPECTED; } }    

    let end = start.elapsed();
    println!("[SGX CLOCK] {}:  {}.{:06} seconds", "central data decryption", end.as_secs(), end.subsec_nanos() / 1_000);


    let dictionary_buffer = EncodedDictionaryBuffer::build_dictionary_buffer(decrypted);
    let mut result_buffer = get_ref_encoded_result_buffer().unwrap().borrow_mut();
    let mut query_buffer = get_ref_encoded_query_buffer().unwrap().borrow_mut();

    dictionary_buffer.intersect(&query_buffer, &mut result_buffer);
    
    sgx_status_t::SGX_SUCCESS
}

// Response construction
#[no_mangle]
pub extern "C" fn get_encoded_result(
    response: *mut u8,
    response_size: usize,
) -> sgx_status_t {
    let result_buffer = get_ref_encoded_result_buffer().unwrap().borrow_mut();
    let query_buffer = get_ref_encoded_query_buffer().unwrap().borrow_mut();
    let mut response_vec: Vec<u8> = Vec::with_capacity(response_size);

    result_buffer.build_query_response(&query_buffer, &mut response_vec);

    /* encryption */
    let mut encrypted_response_vec: Vec<u8> = response_vec.clone();
    for (i, query_rep) in query_buffer.queries.iter().enumerate() {
        let mut counter_block: [u8; 16] = COUNTER_BLOCK;
        let ctr_inc_bits: u32 = SGXSSL_CTR_BITS;

        // Originally shared_key is derived by following Remote Attestation protocol.
        // This is mock of shared key-based encryption.
        let mut shared_key: [u8; 16] = [0; 16];
        shared_key[..8].copy_from_slice(&query_rep.id.to_be_bytes());
        let current_cursor = i*RESPONSE_DATA_SIZE_U8;
        
        // Encrypt only sensitive part, result. query_id should not be encrypted.
        let ret = rsgx_aes_ctr_encrypt(
            &shared_key,
            &response_vec[current_cursor+QUERY_ID_SIZE_U8..current_cursor+RESPONSE_DATA_SIZE_U8],
            &mut counter_block,
            ctr_inc_bits,
            &mut encrypted_response_vec[current_cursor+QUERY_ID_SIZE_U8..current_cursor+RESPONSE_DATA_SIZE_U8]
        );
        match ret { Ok(()) => {}, Err(_) => { return sgx_status_t::SGX_ERROR_UNEXPECTED; } }    
    }

    let slice = encrypted_response_vec.as_mut_slice();
    unsafe {
        for i in 0..response_size {
            *response.offset(i as isize) = slice[i];
        }
    }

    sgx_status_t::SGX_SUCCESS
}
