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

#![crate_name = "helloworldsampleenclave"]
#![crate_type = "staticlib"]

#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

extern crate sgx_types;
extern crate sgx_trts;
#[cfg(not(target_env = "sgx"))]
#[macro_use]
extern crate sgx_tstd as std;

use sgx_types::*;
use std::vec::Vec;
use std::cell::RefCell;
use std::slice;
use std::sync::atomic::{Ordering};
use std::boxed::Box;

mod buffers;
use buffers::*;

#[no_mangle]
pub extern "C" fn upload_query_data(
    total_query_data: * const u8,
    toal_size       : usize,
    size_list       : * const usize,
    client_size     : usize,
    query_id_list   : * const u64,
) -> sgx_status_t {
    println!("[SGX] upload_query_data start");
    
    _init_buffers();

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

    let mut query_buffer = get_ref_query_buffer().unwrap().borrow_mut();
    _build_query_buffer(&mut query_buffer, &total_query_data_vec, &size_list_vec, &query_id_list_vec);

    let mut mapped_query_buffer = get_ref_mapped_query_buffer().unwrap().borrow_mut();
    _map_into_PCT(&mut mapped_query_buffer, &query_buffer);

    
    
    println!("[SGX] upload_query_data succes!");
    sgx_status_t::SGX_SUCCESS
}

fn _init_buffers() {

    // initialize mapped query buffer
    let mut dictionary_buffer = DictionaryBuffer::new();
    let dictionary_buffer_box = Box::new(RefCell::<DictionaryBuffer>::new(dictionary_buffer));
    let dictionary_buffer_ptr = Box::into_raw(dictionary_buffer_box);
    DICTIONARY_BUFFER.store(dictionary_buffer_ptr as *mut (), Ordering::SeqCst);

    // initialize query buffer
    let mut query_buffer = QueryBuffer::new();
    let query_buffer_box = Box::new(RefCell::<QueryBuffer>::new(query_buffer));
    let query_buffer_ptr = Box::into_raw(query_buffer_box);
    QUERY_BUFFER.store(query_buffer_ptr as *mut (), Ordering::SeqCst);

    // initialize mapped query buffer
    let mut mapped_query_buffer = MappedQuery::new();
    let mapped_query_buffer_box = Box::new(RefCell::<MappedQuery>::new(mapped_query_buffer));
    let mapped_query_buffer_ptr = Box::into_raw(mapped_query_buffer_box);
    MAPPED_QUERY_BUFFER.store(mapped_query_buffer_ptr as *mut (), Ordering::SeqCst);

    // initialize mapped query buffer
    let mut result_buffer = DictionaryBuffer::new();
    let result_buffer_box = Box::new(RefCell::<DictionaryBuffer>::new(result_buffer));
    let result_buffer_ptr = Box::into_raw(result_buffer_box);
    RESULT_BUFFER.store(result_buffer_ptr as *mut (), Ordering::SeqCst);
}

// !!このメソッドでは全くerror処理していない
fn _build_query_buffer(
    buffer              : &mut QueryBuffer,
    total_query_data_vec: &Vec<u8>,
    size_list_vec       : &Vec<usize>,
    query_id_list_vec   : &Vec<u64>,
) -> i8 {
    let mut cursor = 0;
    for i in 0_usize..(size_list_vec.len()) {
        let size: usize = size_list_vec[i]*QUERY_U8_SIZE;
        let this_query = &total_query_data_vec[cursor..cursor+size];
        cursor = cursor+size; // 忘れないようにここで更新
        
        let mut query = QueryRep::new();
        query.id = query_id_list_vec[i];
        for i in 0_usize..(size/QUERY_U8_SIZE) {
            let mut timestamp = [0_u8; UNIXEPOCH_U8_SIZE];
            let mut geoHash = [0_u8; GEOHASH_U8_SIZE];
            timestamp.copy_from_slice(&this_query[i*QUERY_U8_SIZE..i*QUERY_U8_SIZE+UNIXEPOCH_U8_SIZE]);
            geoHash.copy_from_slice(&this_query[i*QUERY_U8_SIZE+UNIXEPOCH_U8_SIZE..(i + 1)*QUERY_U8_SIZE]);
            query.parameters.push((geoHash, timestamp));
        }
        buffer.queries.insert(query.id, query);
    }
    return 0;
}

// !!このメソッドでは全くerror処理していない
fn _map_into_PCT(mapped_query_buffer: &mut MappedQuery, query_buffer: &QueryBuffer) -> i8 {
    for query_rep in query_buffer.queries.values() {
        for parameter in query_rep.parameters.iter() {
            match mapped_query_buffer.map.get_mut(&parameter.0) {
                Some(vec) => { vec.push(unixepoch_from_u8(parameter.1)) },
                None => { mapped_query_buffer.map.insert(parameter.0, vec![unixepoch_from_u8(parameter.1)]); },
            };
        }
    }
    return 0;
}

// chunk分割は呼び出し側に任せる
#[no_mangle]
pub extern "C" fn private_contact_trace(
    geohash_u8: * const u8,
    geohash_u8_size: usize,
    unixepoch_u64: * const u64,
    unixepoch_u64_size: usize,
    size_list: * const usize,
    epoch_data_size: usize,
) -> sgx_status_t {
    println!("[SGX] private_contact_trace start");
    let mut rt : sgx_status_t = sgx_status_t::SGX_ERROR_UNEXPECTED;
    let mut dictionary_buffer = get_ref_dictionary_buffer().unwrap().borrow_mut();
    
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

    _build_dictionary_buffer(&mut dictionary_buffer, &geohash_data_vec, &unixepoch_data_vec, &size_list_vec);
    
    println!("[SGX] private_contact_trace succes!");
    sgx_status_t::SGX_SUCCESS
}

fn _build_dictionary_buffer(
    dictionary_buffer: &mut DictionaryBuffer,
    geohash_data_vec: &Vec<u8>,
    unixepoch_data_vec: &Vec<u64>,
    size_list_vec: &Vec<usize>,
) -> i8 {
    let mut cursor: usize = 0;
    for i in 0usize..(size_list_vec.len()) {
        let mut geohash = GeoHashKey::default(); 
        geohash.copy_from_slice(&geohash_data_vec[GEOHASH_U8_SIZE*i..GEOHASH_U8_SIZE*(i+1)]);
        let unixepoch: Vec<UnixEpoch> = unixepoch_data_vec[cursor..cursor+size_list_vec[i]].to_vec();
        dictionary_buffer.data.insert(geohash, unixepoch);
        cursor += size_list_vec[i];
    }
    return 0;
}

// chunk分割は呼び出し側に任せる
#[no_mangle]
pub extern "C" fn get_result(
    total_query_data: * const u8,
    toal_size       : usize,
) -> sgx_status_t {
    println!("[SGX] get_result start");
    let mut rt : sgx_status_t = sgx_status_t::SGX_ERROR_UNEXPECTED;
    
    println!("[SGX] get_result succes!");
    sgx_status_t::SGX_SUCCESS
}