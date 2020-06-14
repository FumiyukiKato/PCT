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


fn main() {
    /* parameters */
    let threashould: usize = 180000;
    
    let q_filename = "data/query/generated-client-query-qs-450-cs-2000-20200614184628.json";
    // let q_filename = "data/query/generated-client-query-qs-4500-cs-1000-20200614022547.json";
    // let q_filename = "data/query/generated-client-query-qs-4500-cs-2000-20200614170915.json";
    // let q_filename = "data/query/generated-client-query-qs-45000-cs-1000-20200614023852.json"; デカすぎ感
    
    // let c_filename = "data/central/generated-central-data-100000-20200614022035.json";
    // let c_filename = "data/central/generated-central-data-1000000-20200614022055.json";
    // let c_filename = "data/central/generated-central-data-10000000-20200614022325.json";
    let c_filename = "data/central/generated-central-data-100000000-20200614030320.json";
    
    let mut clocker = Clocker::new();

    clocker.set_and_start("Read Query Data");
    let query_data = QueryData::read_raw_from_file(q_filename);
    clocker.stop("Read Query Data");

    clocker.set_and_start("Read Central Data");
    let external_data = PCTHash::read_raw_from_file(c_filename);
    clocker.stop("Read Central Data");

    clocker.set_and_start("Distribute central data");
    let mut chunked_buf: Vec<PCTHash> = Vec::with_capacity(threashould);
    external_data.disribute(&mut chunked_buf, threashould);
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

    /* upload query mock data */
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
    let mut chunk_index: usize = 0;
    let last = chunked_buf.len() - 1;
    while last >= chunk_index {

        let chunk = &chunked_buf[chunk_index];
        let mut geohash_u8: Vec<u8> = Vec::with_capacity(threashould*GEOHASH_U8_SIZE);
        let mut unixepoch_u64: Vec<u64> = Vec::with_capacity(threashould);
        let mut size_list: Vec<usize> = Vec::with_capacity(chunk.size());
        let epoch_data_size = chunk.prepare_sgx_data(&mut geohash_u8, &mut unixepoch_u64, &mut size_list);

        let result = unsafe {
            private_contact_trace(
                enclave.geteid(),
                &mut retval,
                geohash_u8.as_ptr() as * const u8,
                geohash_u8.len(),
                unixepoch_u64.as_ptr() as * const u64,
                unixepoch_u64.len(),
                size_list.as_ptr() as * const usize,
                epoch_data_size
            )
        };
        match result {
            sgx_status_t::SGX_SUCCESS => {
                println!("[UNTRUSTED] private_contact_trace Succes! {} th iteration", chunk_index);
            },
            _ => {
                println!("[UNTRUSTED] private_contact_trace Failed {}!", result.as_str());
                return;
            }
        }

        chunk_index += 1;
    }
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
            println!("[UNTRUSTED] get_result Succes!");
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
    println!("[UNTRUSTED] All process is successful!!");
    clocker.show_all();

    let now: String = get_timestamp();
    write_to_file(
        format!("data/result/ex-result-{}.txt", now),
        "simple hash and list".to_string(),
        c_filename.to_string(),
        q_filename.to_string(),
        threashould,
        "only risk_level".to_string(),
        clocker
    );
}
