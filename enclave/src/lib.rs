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

extern crate sgx_fst as fst;

use sgx_types::*;
use std::vec::Vec;
use std::cell::RefCell;
use std::slice;
use std::boxed::Box;
use std::time::{Instant};
use std::untrusted::time::InstantEx;
use std::sync::atomic::{AtomicPtr, Ordering};

mod types;
use types::*;

/* 
SGXのステート
    ステートは全部グローバル変数に持ってヒープにメモリを確保する
*/
pub static QUERY_BUFFER: AtomicPtr<()> = AtomicPtr::new(0 as * mut ());
pub fn get_ref_query_buffer() -> Option<&'static RefCell<QueryBuffer>> {
    let ptr = QUERY_BUFFER.load(Ordering::SeqCst) as * mut RefCell<QueryBuffer>;
    if ptr.is_null() {
        None
    } else {
        Some(unsafe { &* ptr })
    }
}

pub static MAPPED_QUERY_BUFFER: AtomicPtr<()> = AtomicPtr::new(0 as * mut ());
pub fn get_ref_mapped_query_buffer() -> Option<&'static RefCell<MappedQueryBuffer>> {
    let ptr = MAPPED_QUERY_BUFFER.load(Ordering::SeqCst) as * mut RefCell<MappedQueryBuffer>;
    if ptr.is_null() {
        None
    } else {
        Some(unsafe { &* ptr })
    }
}

pub static RESULT_BUFFER: AtomicPtr<()> = AtomicPtr::new(0 as * mut ());
pub fn get_ref_result_buffer() -> Option<&'static RefCell<ResultBuffer>> {
    let ptr = RESULT_BUFFER.load(Ordering::SeqCst) as * mut RefCell<ResultBuffer>;
    if ptr.is_null() {
        None
    } else {
        Some(unsafe { &* ptr })
    }
}

pub static ENCODED_QUERY_BUFFER: AtomicPtr<()> = AtomicPtr::new(0 as * mut ());
pub fn get_ref_encoded_query_buffer() -> Option<&'static RefCell<EncodedQueryBuffer>> {
    let ptr = ENCODED_QUERY_BUFFER.load(Ordering::SeqCst) as * mut RefCell<EncodedQueryBuffer>;
    if ptr.is_null() {
        None
    } else {
        Some(unsafe { &* ptr })
    }
}

pub static MAPPED_ENCODED_QUERY_BUFFER: AtomicPtr<()> = AtomicPtr::new(0 as * mut ());
pub fn get_ref_mapped_encoded_query_buffer() -> Option<&'static RefCell<MappedEncodedQueryBuffer>> {
    let ptr = MAPPED_ENCODED_QUERY_BUFFER.load(Ordering::SeqCst) as * mut RefCell<MappedEncodedQueryBuffer>;
    if ptr.is_null() {
        None
    } else {
        Some(unsafe { &* ptr })
    }
}

#[no_mangle]
pub extern "C" fn upload_query_data(
    total_query_data: *const u8,
    toal_size       : usize,
    size_list       : *const usize,
    client_size     : usize,
    query_id_list   : *const u64,
) -> sgx_status_t {
    // println!("[SGX] upload_query_data start");
    let whole_start = Instant::now();
    let start = Instant::now();
    _init_buffers();
    let end = start.elapsed();
    println!("[SGX CLOCK] {}:  {}.{:06} seconds", "init_buffers", end.as_secs(), end.subsec_nanos() / 1_000);

    let start = Instant::now();
    let total_query_data_vec: Vec<u8> = unsafe {
        slice::from_raw_parts(total_query_data, toal_size)
    }.to_vec();
    if total_query_data_vec.len() != toal_size {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }
    
    let size_list_vec: Vec<usize> = unsafe {
        slice::from_raw_parts(size_list, client_size)
    }.to_vec();
    if size_list_vec.len() != client_size {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }

    let query_id_list_vec: Vec<u64> = unsafe {
        slice::from_raw_parts(query_id_list, client_size)
    }.to_vec();
    if query_id_list_vec.len() != client_size {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }
    let end = start.elapsed();
    println!("[SGX CLOCK] {}:  {}.{:06} seconds", "reading?", end.as_secs(), end.subsec_nanos() / 1_000);

    let start = Instant::now();
    let mut query_buffer = get_ref_query_buffer().unwrap().borrow_mut();
    query_buffer.build_query_buffer(&total_query_data_vec, &size_list_vec, &query_id_list_vec);
    let end = start.elapsed();
    println!("[SGX CLOCK] {}:  {}.{:06} seconds", "build_query_buffer", end.as_secs(), end.subsec_nanos() / 1_000);

    let start = Instant::now();
    let mut mapped_query_buffer = get_ref_mapped_query_buffer().unwrap().borrow_mut();
    mapped_query_buffer.mapping(&query_buffer);
    let end = start.elapsed();
    println!("[SGX CLOCK] {}:  {}.{:06} seconds", "map_into_pct", end.as_secs(), end.subsec_nanos() / 1_000);

    // println!("[SGX] upload_query_data succes!");
    let end = whole_start.elapsed();
    println!("[SGX CLOCK] {}:  {}.{:06} seconds", "whole", end.as_secs(), end.subsec_nanos() / 1_000);
    
    sgx_status_t::SGX_SUCCESS
}

#[no_mangle]
pub extern "C" fn upload_encoded_query_data(
    total_query_data: *const u8,
    total_size       : usize,
    client_size     : usize,
    query_id_list   : *const u64,
) -> sgx_status_t {
    // println!("[SGX] upload_query_data start");
    let whole_start = Instant::now();
    let start = Instant::now();
    _init_encoded_buffers();
    let end = start.elapsed();
    println!("[SGX CLOCK] {}:  {}.{:06} seconds", "init_buffers", end.as_secs(), end.subsec_nanos() / 1_000);

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
    println!("[SGX CLOCK] {}:  {}.{:06} seconds", "reading?", end.as_secs(), end.subsec_nanos() / 1_000);

    let start = Instant::now();
    let mut query_buffer = get_ref_encoded_query_buffer().unwrap().borrow_mut();
    query_buffer.build_query_buffer(&total_query_data_vec, &query_id_list_vec);
    let end = start.elapsed();
    println!("[SGX CLOCK] {}:  {}.{:06} seconds", "build_query_buffer", end.as_secs(), end.subsec_nanos() / 1_000);

    let start = Instant::now();
    let mut mapped_query_buffer = get_ref_mapped_encoded_query_buffer().unwrap().borrow_mut();
    mapped_query_buffer.mapping(&query_buffer);
    let end = start.elapsed();
    println!("[SGX CLOCK] {}:  {}.{:06} seconds", "map_into_pct", end.as_secs(), end.subsec_nanos() / 1_000);

    // println!("[SGX] upload_query_data succes!");
    let end = whole_start.elapsed();
    println!("[SGX CLOCK] {}:  {}.{:06} seconds", "whole", end.as_secs(), end.subsec_nanos() / 1_000);
    
    sgx_status_t::SGX_SUCCESS
}

fn _init_buffers() {

    // initialize query buffer
    let query_buffer = QueryBuffer::new();
    let query_buffer_box = Box::new(RefCell::<QueryBuffer>::new(query_buffer));
    let query_buffer_ptr = Box::into_raw(query_buffer_box);
    QUERY_BUFFER.store(query_buffer_ptr as *mut (), Ordering::SeqCst);

    // initialize mapped query buffer
    let mapped_query_buffer = MappedQueryBuffer::new();
    let mapped_query_buffer_box = Box::new(RefCell::<MappedQueryBuffer>::new(mapped_query_buffer));
    let mapped_query_buffer_ptr = Box::into_raw(mapped_query_buffer_box);
    MAPPED_QUERY_BUFFER.store(mapped_query_buffer_ptr as *mut (), Ordering::SeqCst);

    // initialize result buffer
    let result_buffer = ResultBuffer::new();
    let result_buffer_box = Box::new(RefCell::<ResultBuffer>::new(result_buffer));
    let result_buffer_ptr = Box::into_raw(result_buffer_box);
    RESULT_BUFFER.store(result_buffer_ptr as *mut (), Ordering::SeqCst);
}

fn _init_encoded_buffers() {

    // initialize query buffer
    let query_buffer = EncodedQueryBuffer::new();
    let query_buffer_box = Box::new(RefCell::<EncodedQueryBuffer>::new(query_buffer));
    let query_buffer_ptr = Box::into_raw(query_buffer_box);
    ENCODED_QUERY_BUFFER.store(query_buffer_ptr as *mut (), Ordering::SeqCst);

    // initialize mapped query buffer
    let mapped_query_buffer = MappedEncodedQueryBuffer::new();
    let mapped_query_buffer_box = Box::new(RefCell::<MappedEncodedQueryBuffer>::new(mapped_query_buffer));
    let mapped_query_buffer_ptr = Box::into_raw(mapped_query_buffer_box);
    MAPPED_ENCODED_QUERY_BUFFER.store(mapped_query_buffer_ptr as *mut (), Ordering::SeqCst);

    // initialize result buffer
    let result_buffer = ResultBuffer::new();
    let result_buffer_box = Box::new(RefCell::<ResultBuffer>::new(result_buffer));
    let result_buffer_ptr = Box::into_raw(result_buffer_box);
    RESULT_BUFFER.store(result_buffer_ptr as *mut (), Ordering::SeqCst);
}

/*
ロジック部分
    chunk分割は呼び出し側に任せる
    ResultBufferを作るところまでが責務
*/
#[no_mangle]
pub extern "C" fn private_contact_trace(
    geohash_u8: *const u8,
    geohash_u8_size: usize,
    unixepoch_u64: *const u64,
    unixepoch_u64_size: usize,
    size_list: *const usize,
    epoch_data_size: usize,
) -> sgx_status_t {
    // println!("[SGX] private_contact_trace start");
    let mut dictionary_buffer = DictionaryBuffer::new();

    let geohash_data_vec: Vec<u8> = unsafe {
        slice::from_raw_parts(geohash_u8, geohash_u8_size)
    }.to_vec();
    if geohash_data_vec.len() != geohash_u8_size {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }

    let unixepoch_data_vec: Vec<u64> = unsafe {
        slice::from_raw_parts(unixepoch_u64, unixepoch_u64_size)
    }.to_vec();
    if unixepoch_data_vec.len() != unixepoch_u64_size {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }
    
    let size_list_vec: Vec<usize> = unsafe {
        slice::from_raw_parts(size_list, epoch_data_size)
    }.to_vec();
    if size_list_vec.len() != epoch_data_size {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }

    dictionary_buffer.build_dictionary_buffer(&geohash_data_vec, &unixepoch_data_vec, &size_list_vec);
    let mapped_query_buffer = get_ref_mapped_query_buffer().unwrap().borrow_mut();
    let mut result_buffer = get_ref_result_buffer().unwrap().borrow_mut();
    dictionary_buffer.intersect(&mapped_query_buffer, &mut result_buffer);
    
    // println!("[SGX] private_contact_trace succes!");
    sgx_status_t::SGX_SUCCESS
}

// ResultBufferからQueryResultを組み立てて返す
#[no_mangle]
pub extern "C" fn get_result(
    response: *mut u8,
    response_size: usize,
) -> sgx_status_t {
    // println!("[SGX] get_result start");

    let result_buffer = get_ref_result_buffer().unwrap().borrow_mut();
    let query_buffer = get_ref_query_buffer().unwrap().borrow_mut();
    let mut response_vec: Vec<u8> = Vec::with_capacity(response_size);

    result_buffer.build_query_response(&query_buffer, &mut response_vec);
    let slice = response_vec.as_mut_slice();
    unsafe {
        for i in 0..response_size {
            *response.offset(i as isize) = slice[i];
        }
    }
    
    // println!("[SGX] get_result succes!");
    sgx_status_t::SGX_SUCCESS
}
