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
    total_query_data: *const u8,
    toal_size       : usize,
    size_list       : *const usize,
    client_size     : usize,
    query_id_list   : *const u64,
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
    _map_into_pct(&mut mapped_query_buffer, &query_buffer);

    println!("[SGX] upload_query_data succes!");
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

// !!このメソッドでは全くerror処理していない
// queryを個々に組み立ててbufferに保持する
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
            let mut geo_hash = [0_u8; GEOHASH_U8_SIZE];
            timestamp.copy_from_slice(&this_query[i*QUERY_U8_SIZE..i*QUERY_U8_SIZE+UNIXEPOCH_U8_SIZE]);
            geo_hash.copy_from_slice(&this_query[i*QUERY_U8_SIZE+UNIXEPOCH_U8_SIZE..(i + 1)*QUERY_U8_SIZE]);
            match query.parameters.get_mut(&geo_hash) {
                Some(sorted_list) => { _sorted_push(sorted_list, unixepoch_from_u8(timestamp)) },
                None => { query.parameters.insert(geo_hash as GeoHashKey, vec![unixepoch_from_u8(timestamp)]); },
            };
        }
        buffer.queries.push(query);
    }
    return 0;
}

// !!このメソッドでは全くerror処理していない
fn _map_into_pct(mapped_query_buffer: &mut MappedQueryBuffer, query_buffer: &QueryBuffer) -> i8 {
    for query_rep in query_buffer.queries.iter() {
        for (geohash, unixepoch_vec) in query_rep.parameters.iter() {
            match mapped_query_buffer.map.get(geohash) {
                Some(sorted_list) => { mapped_query_buffer.map.insert(*geohash, _sorted_merge(&sorted_list, unixepoch_vec)); },
                None => { mapped_query_buffer.map.insert(*geohash, unixepoch_vec.to_vec()); },
            };
        }
    }
    return 0;
}

// 昇順ソート+ユニーク性
// あえてジェネリクスにする必要はない，むしろ型で守っていく
// Vecだと遅いけどLinkedListよりはキャッシュに乗るので早い気がするのでVecでいく
fn _sorted_push(sorted_list: &mut Vec<UnixEpoch>, unixepoch: UnixEpoch) {
    let mut index = 0;
    for elm in sorted_list.iter() {
        if *elm > unixepoch {
            sorted_list.insert(index, unixepoch);
            return;
        } else if *elm == unixepoch {
            return;
        } else {
            index += 1;
        }
    }
    sorted_list.push(unixepoch);
}

// なんかめっちゃ長くなってしまったけど
fn _sorted_merge(sorted_list_1: &Vec<UnixEpoch>, sorted_list_2: &Vec<UnixEpoch>) -> Vec<UnixEpoch> {
    let len1 = sorted_list_1.len();
    let len2 = sorted_list_2.len();
    let size = len1 + len2;
    let mut cursor1 = 0;
    let mut cursor2 = 0;
    let mut tmp_max = 0;

    let mut merged_vec = Vec::with_capacity(size);
    for _ in 0..size {
        let mut candidate = if sorted_list_1[cursor1] < sorted_list_2[cursor2] {
            cursor1 += 1;
            sorted_list_1[cursor1 - 1]
        } else if sorted_list_1[cursor1] == sorted_list_2[cursor2] {
            cursor1 += 1;
            cursor2 += 1;
            sorted_list_1[cursor1 - 1]
        } else {
            cursor2 += 1;
            sorted_list_2[cursor2 - 1]
        };
        
        if tmp_max != candidate { 
            tmp_max = candidate; 
            merged_vec.push(tmp_max);
        }
        
        if len1 == cursor1 && cursor2 < len2 {
            for j in cursor2..len2 {
                candidate = sorted_list_2[j];
                if tmp_max != candidate { tmp_max = candidate; } else { continue; };
                merged_vec.push(candidate);
            }
            break;
        }
        if len2 == cursor2 && cursor1 < len1 {
            for j in cursor1..len1 {
                candidate = sorted_list_1[j];
                if tmp_max != candidate { tmp_max = candidate; } else { continue; };
                merged_vec.push(candidate);
            }
            break;
        }
        if len1 == cursor1 && len2 == cursor2 { break; }
    }
    merged_vec
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

    _build_dictionary_buffer(&mut dictionary_buffer, &geohash_data_vec, &unixepoch_data_vec, &size_list_vec);
    let mapped_query_buffer = get_ref_mapped_query_buffer().unwrap().borrow_mut();
    let mut result_buffer = get_ref_result_buffer().unwrap().borrow_mut();
    dictionary_buffer.intersect(&mapped_query_buffer, &mut result_buffer);
    
    // println!("[SGX] private_contact_trace succes!");
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
        // centralデータはすでにsorted unique listになっている
        let unixepoch: Vec<UnixEpoch> = unixepoch_data_vec[cursor..cursor+size_list_vec[i]].to_vec();
        dictionary_buffer.data.insert(geohash, unixepoch);
        cursor += size_list_vec[i];
    }
    return 0;
}

// ResultBufferからQueryResultを組み立てて返す
#[no_mangle]
pub extern "C" fn get_result(
    response: *mut u8,
    response_size: usize,
) -> sgx_status_t {
    println!("[SGX] get_result start");

    let result_buffer = get_ref_result_buffer().unwrap().borrow_mut();
    let query_buffer = get_ref_query_buffer().unwrap().borrow_mut();
    let mut response_vec: Vec<u8> = Vec::with_capacity(response_size);

    _build_query_response(&result_buffer, &query_buffer, &mut response_vec);
    let slice = response_vec.as_mut_slice();
    unsafe {
        for i in 0..response_size {
            *response.offset(i as isize) = slice[i];
        }
    }
    
    println!("[SGX] get_result succes!");
    sgx_status_t::SGX_SUCCESS
}

// matchがネストして読みにくくなってしまっている
// メソッドチェーンでもっと関数型っぽく書けば読みやすくなりそうではある
fn _build_query_response(
    result_buffer: &ResultBuffer,
    query_buffer: &QueryBuffer,
    response_vec: &mut Vec<u8>,
) {
    for query in query_buffer.queries.iter() {
        let mut result = QueryResult::new();
        result.query_id = query.id;
        for (geohash, unixepoch) in result_buffer.data.iter() {
            let is_exist = match query.parameters.get(geohash) {
                Some(sorted_list) => { 
                    match sorted_list.binary_search(unixepoch) {
                        Ok(_) => true,
                        Err(_) => false,
                    }
                },
                None => { false },
            };
            if is_exist {
                result.result_vec.push((*geohash, *unixepoch));
            }
        }
        result.risk_level = match result.result_vec.len() {
            x if x > 20 => 3,
            x if x > 5 => 2,
            x if x > 0  => 1,
            _ => 0,
        };
        response_vec.extend_from_slice(&result.to_be_bytes());
    }
}