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

extern crate sgx_types;
extern crate sgx_urts;
extern crate serde;
extern crate serde_json;
use std::env;

use sgx_types::*;

mod query_data;
use query_data::*;

// ecallsはnamedで呼び出す
mod ecalls;
use ecalls::{upload_query_data, init_enclave, private_contact_trace, get_result};

mod central_data;
use central_data::*;

mod util;
use util::*;

const RESPONSE_DATA_SIZE_U8: usize = 9;


/*
    args[0] = threashold
    args[1] = query data file path
    args[2] = central data file path
    args[3] = result file output (false or true)
*/
fn _get_options() -> Vec<String> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() != 4 {
        println!("[UNTRUSTED] ERROR bin/app needs 4 arguments!");
        println!("    args[0] = threashold");
        println!("    args[1] = query data file path");
        println!("    args[2] = central data file path");
        println!("    args[3] = result file output (false or true)");
        std::process::exit(-1);
    }
    args
}

fn main() {
    baselineNoQueryMulitiplexingAndNoChunk();
}

// ハッシュテーブル
fn geohashTable() {
    let args = _get_options();
    /* parameters */
    let threashould: usize = args[0].parse().unwrap();
    let q_filename = &args[1];
    let c_filename = &args[2];

    let mut clocker = Clocker::new();

    /* read central data */
    clocker.set_and_start("Read Central Data");
    let external_data = GeohashTable::read_raw_from_file(c_filename);
    clocker.stop("Read Central Data");

    /* preprocess central data */
    clocker.set_and_start("Distribute central data");
    let mut chunked_buf: Vec<GeohashTable> = Vec::with_capacity(threashould);
    external_data.disribute(&mut chunked_buf, threashould);
    let mut sgx_data: Vec<(Vec<u8>, Vec<u64>, Vec<usize>, usize)> = Vec::with_capacity(100);
    let mut chunk_curret_index: usize = 0;
    let chunk_last_index = chunked_buf.len() - 1;
    while chunk_last_index >= chunk_curret_index {
        let chunk = &chunked_buf[chunk_curret_index];
        let mut geohash_u8: Vec<u8> = Vec::with_capacity(threashould*GEOHASH_U8_SIZE);
        let mut unixepoch_u64: Vec<u64> = Vec::with_capacity(threashould);
        let mut size_list: Vec<usize> = Vec::with_capacity(chunk.size());
        let epoch_data_size = chunk.prepare_sgx_data(&mut geohash_u8, &mut unixepoch_u64, &mut size_list);
        sgx_data.push((geohash_u8, unixepoch_u64, size_list, epoch_data_size));
        chunk_curret_index += 1;
    }
    clocker.stop("Distribute central data");

    /* initialize enclave */
    clocker.set_and_start("ECALL init_enclave");
    let enclave = match init_enclave() {
        Ok(r) => {
            println!("[UNTRUSTED] Init Enclave Successful {}!", r.geteid());
            r
        },
        Err(x) => {
            println!("[UNTRUSTED] Init Enclave Failed {}!", x.as_str());
            return;
        },
    };
    clocker.stop("ECALL init_enclave");

    /* read query data */
    clocker.set_and_start("Read Query Data");
    let query_data = QueryData::read_raw_from_file(q_filename);
    clocker.stop("Read Query Data");

    /* upload query data */
    clocker.set_and_start("ECALL upload_query_data");
    let mut retval = sgx_status_t::SGX_SUCCESS;
    let result = unsafe {
        upload_query_data(
            enclave.geteid(),
            &mut retval,
            query_data.total_data_to_u8().as_ptr() as * const u8,
            query_data.total_size(),
            query_data.size_list().as_ptr() as * const usize,
            query_data.client_size,
            query_data.query_id_list().as_ptr() as * const u64
        )
    };
    match result {
        sgx_status_t::SGX_SUCCESS => {
            println!("[UNTRUSTED] upload_query_data Succes!");
        },
        _ => {
            println!("[UNTRUSTED] upload_query_data Failed {}!", result.as_str());
            return;
        }
    }
    clocker.stop("ECALL upload_query_data");

    /* main logic contact tracing */
    let mut chunk_index: usize = 0;
    let last = chunked_buf.len() - 1;
    clocker.set_and_start("ECALL private_contact_trace");
    while last >= chunk_index {

        let chunk = &sgx_data[chunk_index];
        let result = unsafe {
            private_contact_trace(
                enclave.geteid(),
                &mut retval,
                chunk.0.as_ptr() as * const u8,
                chunk.0.len(),
                chunk.1.as_ptr() as * const u64,
                chunk.1.len(),
                chunk.2.as_ptr() as * const usize,
                chunk.3
            )
        };
        match result {
            sgx_status_t::SGX_SUCCESS => {
                print!("\r[UNTRUSTED] private_contact_trace Succes! {} th iteration", chunk_index);
            },
            _ => {
                println!("[UNTRUSTED] private_contact_trace Failed {}!", result.as_str());
                return;
            }
        }
        chunk_index += 1;
    }
    println!("");
    
    clocker.stop("ECALL private_contact_trace");

    /* response reconstruction */
    clocker.set_and_start("ECALL get_result");
    let response_size = query_data.client_size * RESPONSE_DATA_SIZE_U8;
    let mut response: Vec<u8> = vec![0; response_size];
    let result = unsafe {
        get_result(
            enclave.geteid(),
            &mut retval,
            response.as_mut_ptr(),
            response_size
        )
    };
    match result {
        sgx_status_t::SGX_SUCCESS => {
            // println!("[UNTRUSTED] get_result Succes!");
        },
        _ => {
            println!("[UNTRUSTED] get_result Failed {}!", result.as_str());
            return;
        }
    }
    clocker.stop("ECALL get_result");
    
    for i in 0..query_data.client_size {
        if response[i*RESPONSE_DATA_SIZE_U8+8] == 1 {
            println!("[UNTRUSTED] positive result queryId: {}, {}", query_id_from_u8(&response[i*RESPONSE_DATA_SIZE_U8..i*RESPONSE_DATA_SIZE_U8+8]), response[i*RESPONSE_DATA_SIZE_U8+8]);
        }
    }

    /* finish */
    enclave.destroy();
    // println!("[UNTRUSTED] All process is successful!!");
    clocker.show_all();
    if args[3] == "true".to_string() {
        let now: String = get_timestamp();
        write_to_file(
            format!("data/result/result-{}-geohashTable.txt", now),
            "simple hash and list".to_string(),
            c_filename.to_string(),
            q_filename.to_string(),
            threashould,
            "only risk_level".to_string(),
            clocker
        );
    }
}

// ハッシュテーブル + Periodデータ
fn geohashTableWithPeriodArray() {
    let args = _get_options();
    /* parameters */
    let threashould: usize = args[0].parse().unwrap();
    let q_filename = &args[1];
    let c_filename = &args[2];

    let mut clocker = Clocker::new();

    /* read central data */
    clocker.set_and_start("Read Central Data");
    let external_data = GeohashTableWithPeriodArray::read_raw_from_file(c_filename);
    clocker.stop("Read Central Data");

    /* preprocess central data */
    clocker.set_and_start("Distribute central data");
    let mut chunked_buf: Vec<GeohashTableWithPeriodArray> = Vec::with_capacity(threashould);
    external_data.disribute(&mut chunked_buf, threashould);
    let mut sgx_data: Vec<(Vec<u8>, Vec<u64>, Vec<usize>, usize)> = Vec::with_capacity(100);
    let mut chunk_curret_index: usize = 0;
    let chunk_last_index = chunked_buf.len() - 1;
    while chunk_last_index >= chunk_curret_index {
        let chunk = &chunked_buf[chunk_curret_index];
        let mut geohash_u8: Vec<u8> = Vec::with_capacity(threashould*GEOHASH_U8_SIZE);
        let mut period_u64: Vec<u64> = Vec::with_capacity(threashould*2);
        let mut size_list: Vec<usize> = Vec::with_capacity(chunk.size());
        let epoch_data_size = chunk.prepare_sgx_data(&mut geohash_u8, &mut period_u64, &mut size_list);
        sgx_data.push((geohash_u8, period_u64, size_list, epoch_data_size));
        chunk_curret_index += 1;
    }
    clocker.stop("Distribute central data");

    /* initialize enclave */
    clocker.set_and_start("ECALL init_enclave");
    let enclave = match init_enclave() {
        Ok(r) => {
            println!("[UNTRUSTED] Init Enclave Successful {}!", r.geteid());
            r
        },
        Err(x) => {
            println!("[UNTRUSTED] Init Enclave Failed {}!", x.as_str());
            return;
        },
    };
    clocker.stop("ECALL init_enclave");

    /* read query data */
    clocker.set_and_start("Read Query Data");
    let query_data = QueryData::read_raw_from_file(q_filename);
    clocker.stop("Read Query Data");

    /* upload query data */
    clocker.set_and_start("ECALL upload_query_data");
    let mut retval = sgx_status_t::SGX_SUCCESS;
    let result = unsafe {
        upload_query_data(
            enclave.geteid(),
            &mut retval,
            query_data.total_data_to_u8().as_ptr() as * const u8,
            query_data.total_size(),
            query_data.size_list().as_ptr() as * const usize,
            query_data.client_size,
            query_data.query_id_list().as_ptr() as * const u64
        )
    };
    match result {
        sgx_status_t::SGX_SUCCESS => {
            println!("[UNTRUSTED] upload_query_data Succes!");
        },
        _ => {
            println!("[UNTRUSTED] upload_query_data Failed {}!", result.as_str());
            return;
        }
    }
    clocker.stop("ECALL upload_query_data");

    /* main logic contact tracing */
    let mut chunk_index: usize = 0;
    let last = chunked_buf.len() - 1;
    clocker.set_and_start("ECALL private_contact_trace");
    while last >= chunk_index {

        let chunk = &sgx_data[chunk_index];
        let result = unsafe {
            private_contact_trace(
                enclave.geteid(),
                &mut retval,
                chunk.0.as_ptr() as * const u8,
                chunk.0.len(),
                chunk.1.as_ptr() as * const u64,
                chunk.1.len(),
                chunk.2.as_ptr() as * const usize,
                chunk.3
            )
        };
        match result {
            sgx_status_t::SGX_SUCCESS => {
                print!("\r[UNTRUSTED] private_contact_trace Succes! {} th iteration", chunk_index);
            },
            _ => {
                println!("[UNTRUSTED] private_contact_trace Failed {}!", result.as_str());
                return;
            }
        }
        chunk_index += 1;
    }
    println!("");
    
    clocker.stop("ECALL private_contact_trace");

    /* response reconstruction */
    clocker.set_and_start("ECALL get_result");
    let response_size = query_data.client_size * RESPONSE_DATA_SIZE_U8;
    let mut response: Vec<u8> = vec![0; response_size];
    let result = unsafe {
        get_result(
            enclave.geteid(),
            &mut retval,
            response.as_mut_ptr(),
            response_size
        )
    };
    match result {
        sgx_status_t::SGX_SUCCESS => {
            // println!("[UNTRUSTED] get_result Succes!");
        },
        _ => {
            println!("[UNTRUSTED] get_result Failed {}!", result.as_str());
            return;
        }
    }
    clocker.stop("ECALL get_result");
    
    for i in 0..query_data.client_size {
        if response[i*RESPONSE_DATA_SIZE_U8+8] == 1 {
            println!("[UNTRUSTED] positive result queryId: {}, {}", query_id_from_u8(&response[i*RESPONSE_DATA_SIZE_U8..i*RESPONSE_DATA_SIZE_U8+8]), response[i*RESPONSE_DATA_SIZE_U8+8]);
        }
    }

    /* finish */
    enclave.destroy();
    // println!("[UNTRUSTED] All process is successful!!");
    clocker.show_all();
    if args[3] == "true".to_string() {
        let now: String = get_timestamp();
        write_to_file(
            format!("data/result/result-{}-geohashTableWithPeriodArray.txt", now),
            "simple hash and list".to_string(),
            c_filename.to_string(),
            q_filename.to_string(),
            threashould,
            "only risk_level".to_string(),
            clocker
        );
    }
}

// ベースライン チャンク化なし
fn baselineNoChunk() {
    let args = _get_options();
    /* parameters */
    let threashould: usize = args[0].parse().unwrap();
    let q_filename = &args[1];
    let c_filename = &args[2];

    let mut clocker = Clocker::new();

    /* read central data */
    clocker.set_and_start("Read Central Data");
    let external_data = PlainTable::read_raw_from_file(c_filename);
    clocker.stop("Read Central Data");

    /* preprocess central data 
        チャンク化しないでセットする
    */
    clocker.set_and_start("Prepare central data");
    let mut geohash_u8: Vec<u8> = Vec::with_capacity(100000);
    let mut unixepoch_u64: Vec<u64> = Vec::with_capacity(100000);
    let mut size_list: Vec<usize> = Vec::with_capacity(100000);
    let data_size = external_data.prepare_sgx_data(&mut geohash_u8, &mut unixepoch_u64, &mut size_list);
    clocker.stop("Prepare central data");

    /* initialize enclave */
    clocker.set_and_start("ECALL init_enclave");
    let enclave = match init_enclave() {
        Ok(r) => {
            println!("[UNTRUSTED] Init Enclave Successful {}!", r.geteid());
            r
        },
        Err(x) => {
            println!("[UNTRUSTED] Init Enclave Failed {}!", x.as_str());
            return;
        },
    };
    clocker.stop("ECALL init_enclave");

    /* read query data */
    clocker.set_and_start("Read Query Data");
    let query_data = QueryData::read_raw_from_file(q_filename);
    clocker.stop("Read Query Data");

    /* upload query data */
    clocker.set_and_start("ECALL upload_query_data");
    let mut retval = sgx_status_t::SGX_SUCCESS;
    let result = unsafe {
        upload_query_data(
            enclave.geteid(),
            &mut retval,
            query_data.total_data_to_u8().as_ptr() as * const u8,
            query_data.total_size(),
            query_data.size_list().as_ptr() as * const usize,
            query_data.client_size,
            query_data.query_id_list().as_ptr() as * const u64
        )
    };
    match result {
        sgx_status_t::SGX_SUCCESS => {
            println!("[UNTRUSTED] upload_query_data Succes!");
        },
        _ => {
            println!("[UNTRUSTED] upload_query_data Failed {}!", result.as_str());
            return;
        }
    }
    clocker.stop("ECALL upload_query_data");

    /* main logic contact tracing */
    clocker.set_and_start("ECALL private_contact_trace");
    let result = unsafe {
        private_contact_trace(
            enclave.geteid(),
            &mut retval,
            geohash_u8.as_ptr() as * const u8,
            geohash_u8.len(),
            unixepoch_u64.as_ptr() as * const u64,
            unixepoch_u64.len(),
            size_list.as_ptr() as * const usize,
            data_size
        )
    };
    match result {
        sgx_status_t::SGX_SUCCESS => {
            print!("\r[UNTRUSTED] private_contact_trace Succes!");
        },
        _ => {
            println!("[UNTRUSTED] private_contact_trace Failed {}!", result.as_str());
            return;
        }
    }
    println!("");
    clocker.stop("ECALL private_contact_trace");

    /* response reconstruction */
    clocker.set_and_start("ECALL get_result");
    let response_size = query_data.client_size * RESPONSE_DATA_SIZE_U8;
    let mut response: Vec<u8> = vec![0; response_size];
    let result = unsafe {
        get_result(
            enclave.geteid(),
            &mut retval,
            response.as_mut_ptr(),
            response_size
        )
    };
    match result {
        sgx_status_t::SGX_SUCCESS => {
            // println!("[UNTRUSTED] get_result Succes!");
        },
        _ => {
            println!("[UNTRUSTED] get_result Failed {}!", result.as_str());
            return;
        }
    }
    clocker.stop("ECALL get_result");
    
    for i in 0..query_data.client_size {
        if response[i*RESPONSE_DATA_SIZE_U8+8] == 1 {
            println!("[UNTRUSTED] positive result queryId: {}, {}", query_id_from_u8(&response[i*RESPONSE_DATA_SIZE_U8..i*RESPONSE_DATA_SIZE_U8+8]), response[i*RESPONSE_DATA_SIZE_U8+8]);
        }
    }

    /* finish */
    enclave.destroy();
    // println!("[UNTRUSTED] All process is successful!!");
    clocker.show_all();
    if args[3] == "true".to_string() {
        let now: String = get_timestamp();
        write_to_file(
            format!("data/result/result-{}-baselineNoChunk.txt", now),
            "simple hash and list".to_string(),
            c_filename.to_string(),
            q_filename.to_string(),
            threashould,
            "only risk_level".to_string(),
            clocker
        );
    }
}

// cuckoo ハッシュ やる意味があまりなさそう
// サークルゲームで辞書表現を使用している中だとFPRが認められないので，，，
fn cuckooHasing() {
    let args = _get_options();
    /* parameters */
    let threashould: usize = args[0].parse().unwrap();
    let q_filename = &args[1];
    let c_filename = &args[2];

    let mut clocker = Clocker::new();

    /* read central data */
    clocker.set_and_start("Read Central Data");
    let external_data = GeohashTable::read_raw_from_file(c_filename);
    clocker.stop("Read Central Data");

    /* preprocess central data */
    clocker.set_and_start("Distribute central data");
    let mut chunked_buf: Vec<GeohashTable> = Vec::with_capacity(threashould);
    external_data.disribute(&mut chunked_buf, threashould);
    let mut sgx_data: Vec<(Vec<u8>, Vec<u64>, Vec<usize>, usize)> = Vec::with_capacity(100);
    let mut chunk_curret_index: usize = 0;
    let chunk_last_index = chunked_buf.len() - 1;
    while chunk_last_index >= chunk_curret_index {
        let chunk = &chunked_buf[chunk_curret_index];
        let mut geohash_u8: Vec<u8> = Vec::with_capacity(threashould*GEOHASH_U8_SIZE);
        let mut unixepoch_u64: Vec<u64> = Vec::with_capacity(threashould);
        let mut size_list: Vec<usize> = Vec::with_capacity(chunk.size());
        let epoch_data_size = chunk.prepare_sgx_data(&mut geohash_u8, &mut unixepoch_u64, &mut size_list);
        sgx_data.push((geohash_u8, unixepoch_u64, size_list, epoch_data_size));
        chunk_curret_index += 1;
    }
    clocker.stop("Distribute central data");

    /* initialize enclave */
    clocker.set_and_start("ECALL init_enclave");
    let enclave = match init_enclave() {
        Ok(r) => {
            println!("[UNTRUSTED] Init Enclave Successful {}!", r.geteid());
            r
        },
        Err(x) => {
            println!("[UNTRUSTED] Init Enclave Failed {}!", x.as_str());
            return;
        },
    };
    clocker.stop("ECALL init_enclave");

    /* read query data */
    clocker.set_and_start("Read Query Data");
    let query_data = QueryData::read_raw_from_file(q_filename);
    clocker.stop("Read Query Data");

    /* upload query data */
    clocker.set_and_start("ECALL upload_query_data");
    let mut retval = sgx_status_t::SGX_SUCCESS;
    let result = unsafe {
        upload_query_data(
            enclave.geteid(),
            &mut retval,
            query_data.total_data_to_u8().as_ptr() as * const u8,
            query_data.total_size(),
            query_data.size_list().as_ptr() as * const usize,
            query_data.client_size,
            query_data.query_id_list().as_ptr() as * const u64
        )
    };
    match result {
        sgx_status_t::SGX_SUCCESS => {
            println!("[UNTRUSTED] upload_query_data Succes!");
        },
        _ => {
            println!("[UNTRUSTED] upload_query_data Failed {}!", result.as_str());
            return;
        }
    }
    clocker.stop("ECALL upload_query_data");

    /* main logic contact tracing */
    let mut chunk_index: usize = 0;
    let last = chunked_buf.len() - 1;
    clocker.set_and_start("ECALL private_contact_trace");
    while last >= chunk_index {

        let chunk = &sgx_data[chunk_index];
        let result = unsafe {
            private_contact_trace(
                enclave.geteid(),
                &mut retval,
                chunk.0.as_ptr() as * const u8,
                chunk.0.len(),
                chunk.1.as_ptr() as * const u64,
                chunk.1.len(),
                chunk.2.as_ptr() as * const usize,
                chunk.3
            )
        };
        match result {
            sgx_status_t::SGX_SUCCESS => {
                print!("\r[UNTRUSTED] private_contact_trace Succes! {} th iteration", chunk_index);
            },
            _ => {
                println!("[UNTRUSTED] private_contact_trace Failed {}!", result.as_str());
                return;
            }
        }
        chunk_index += 1;
    }
    println!("");
    
    clocker.stop("ECALL private_contact_trace");

    /* response reconstruction */
    clocker.set_and_start("ECALL get_result");
    let response_size = query_data.client_size * RESPONSE_DATA_SIZE_U8;
    let mut response: Vec<u8> = vec![0; response_size];
    let result = unsafe {
        get_result(
            enclave.geteid(),
            &mut retval,
            response.as_mut_ptr(),
            response_size
        )
    };
    match result {
        sgx_status_t::SGX_SUCCESS => {
            // println!("[UNTRUSTED] get_result Succes!");
        },
        _ => {
            println!("[UNTRUSTED] get_result Failed {}!", result.as_str());
            return;
        }
    }
    clocker.stop("ECALL get_result");
    
    for i in 0..query_data.client_size {
        if response[i*RESPONSE_DATA_SIZE_U8+8] == 1 {
            println!("[UNTRUSTED] positive result queryId: {}, {}", query_id_from_u8(&response[i*RESPONSE_DATA_SIZE_U8..i*RESPONSE_DATA_SIZE_U8+8]), response[i*RESPONSE_DATA_SIZE_U8+8]);
        }
    }

    /* finish */
    enclave.destroy();
    // println!("[UNTRUSTED] All process is successful!!");
    clocker.show_all();
    if args[3] == "true".to_string() {
        let now: String = get_timestamp();
        write_to_file(
            format!("data/result/result-{}-cuckooHasing.txt", now),
            "simple hash and list".to_string(),
            c_filename.to_string(),
            q_filename.to_string(),
            threashould,
            "only risk_level".to_string(),
            clocker
        );
    }
}