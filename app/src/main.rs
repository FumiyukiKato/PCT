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
extern crate fst;
use std::env;

use sgx_types::*;

mod query_data;
use query_data::*;

// ecallsはnamedで呼び出す
mod ecalls;
use ecalls::{ 
    upload_encoded_query_data, 
    init_enclave,
    private_encode_contact_trace, get_encoded_result
};

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
    finiteStateTranducer();

    // use fst::raw::{Fst};
    // use fst::{Set};

    // #[derive(Debug)]
    // struct U8 { vec: Vec<u8> }

    // impl U8 {
    //         fn from_vec(value: Vec<u8>) -> Self {
    //                 U8 { vec: value }
    //         }
    // }

    // impl AsRef<[u8]> for U8 {
    //     #[inline]
    //     fn as_ref(&self) -> &[u8] {
    //         &self.vec
    //     }
    // }

    // // A convenient way to create sets in memory.
    // let mut keys = vec![];
    // keys.push(U8::from_vec([1,2,3,4,5].to_vec()));
    // keys.push(U8::from_vec([1,2,3,4,5].to_vec()));
    // keys.push(U8::from_vec([10,2,30,40,60].to_vec()));
    // keys.push(U8::from_vec([10,15,30,40,60].to_vec()));
    // keys.push(U8::from_vec([10,21,30,40,60,1,1,5,6,6,4].to_vec()));
    // keys.push(U8::from_vec([10,21,30,40,60,1,2,3,4,5].to_vec()));
    // let set = Set::from_iter(keys).unwrap();
    // println!("set {:?}", set);

    // println!("{}", set.contains([10,21,30,40,60,1,2,3,4,5]));
    // println!("{}", set.contains([10,21,30,40,60,1,1,5,6,6,4]));
    // println!("{}", set.contains([10,21,30,40,60,1]));

    // let bytes = set.as_ref().as_bytes().to_vec();
    // let new_set = Set::from_bytes(bytes);
    // println!("new_set {:?}", new_set);

    // println!("{}", set.contains([10,21,30,40,60,1,2,3,4,5]));
    // println!("{}", set.contains([10,21,30,40,60,1,1,5,6,6,4]));
    // println!("{}", set.contains([10,21,30,40,60,1]));
}

fn encodedHasing() {
    let args = _get_options();
    /* parameters */
    let threashould: usize = args[0].parse().unwrap();
    let q_filename = &args[1];
    let c_filename = &args[2];

    let mut clocker = Clocker::new();

    /* read central data */
    clocker.set_and_start("Read Central Data");
    let external_data = EncodedData::read_raw_from_file(c_filename);
    clocker.stop("Read Central Data");

    /* preprocess central data */
    clocker.set_and_start("Distribute central data");
    let mut chunked_buf: Vec<EncodedData> = Vec::with_capacity(threashould);
    external_data.disribute(&mut chunked_buf, threashould);
    let mut sgx_data: Vec<(Vec<u8>, usize)> = Vec::with_capacity(100);
    let mut chunk_curret_index: usize = 0;
    let chunk_last_index = chunked_buf.len() - 1;
    while chunk_last_index >= chunk_curret_index {
        let chunk = &chunked_buf[chunk_curret_index];
        let mut encoded_value_u8: Vec<u8> = Vec::with_capacity(threashould*14);
        let epoch_data_size = chunk.prepare_sgx_data(&mut encoded_value_u8);
        sgx_data.push((encoded_value_u8, epoch_data_size));
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
    let query_data = EncodedQueryData::read_raw_from_file(q_filename);
    clocker.stop("Read Query Data");

    /* upload query data */
    clocker.set_and_start("ECALL upload_query_data");
    let mut retval = sgx_status_t::SGX_SUCCESS;
    let result = unsafe {
        upload_encoded_query_data(
            enclave.geteid(),
            &mut retval,
            query_data.total_data_to_u8().as_ptr() as * const u8,
            query_data.total_size(),
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
            private_encode_contact_trace(
                enclave.geteid(),
                &mut retval,
                chunk.0.as_ptr() as * const u8,
                chunk.0.len(),
                chunk.1
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
        get_encoded_result(
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
    
    let mut positive_queries = vec![];
    for i in 0..query_data.client_size {
        if response[i*RESPONSE_DATA_SIZE_U8+8] == 1 {
            positive_queries.push(query_id_from_u8(&response[i*RESPONSE_DATA_SIZE_U8..i*RESPONSE_DATA_SIZE_U8+8]));
        }
    }
    println!("[UNTRUSTED] positive result queryIds: {:?}", positive_queries);

    /* finish */
    enclave.destroy();
    // println!("[UNTRUSTED] All process is successful!!");
    clocker.show_all();
    if args[3] == "true".to_string() {
        let now: String = get_timestamp();
        write_to_file(
            format!("data/result/result-{}-encodedHasing.txt", now),
            "simple hash and list".to_string(),
            c_filename.to_string(),
            q_filename.to_string(),
            threashould,
            "only risk_level".to_string(),
            clocker
        );
    }
}

fn finiteStateTranducer() {
    let args = _get_options();
    /* parameters */
    let threashould: usize = args[0].parse().unwrap();
    let q_filename = &args[1];
    let c_filename = &args[2];

    let mut clocker = Clocker::new();

    /* read central data */
    clocker.set_and_start("Read Central Data");
    let external_data = EncodedData::read_raw_from_file(c_filename);
    clocker.stop("Read Central Data");

    /* preprocess central data */
    clocker.set_and_start("Distribute central data");
    let mut chunked_buf: Vec<EncodedData> = Vec::with_capacity(threashould);
    external_data.disribute(&mut chunked_buf, threashould);
    let mut sgx_data: Vec<(Vec<u8>, usize)> = Vec::with_capacity(100);
    let mut chunk_curret_index: usize = 0;
    let chunk_last_index = chunked_buf.len() - 1;
    while chunk_last_index >= chunk_curret_index {
        let chunk = &chunked_buf[chunk_curret_index];
        let mut encoded_value_u8: Vec<u8> = Vec::with_capacity(threashould*14);
        let epoch_data_size = chunk.prepare_sgx_data(&mut encoded_value_u8);
        sgx_data.push((encoded_value_u8, epoch_data_size));
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
    let query_data = EncodedQueryData::read_raw_from_file(q_filename);
    clocker.stop("Read Query Data");

    /* upload query data */
    clocker.set_and_start("ECALL upload_query_data");
    let mut retval = sgx_status_t::SGX_SUCCESS;
    let result = unsafe {
        upload_encoded_query_data(
            enclave.geteid(),
            &mut retval,
            query_data.total_data_to_u8().as_ptr() as * const u8,
            query_data.total_size(),
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
            private_encode_contact_trace(
                enclave.geteid(),
                &mut retval,
                chunk.0.as_ptr() as * const u8,
                chunk.0.len(),
                chunk.1
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
        get_encoded_result(
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
    
    let mut positive_queries = vec![];
    for i in 0..query_data.client_size {
        if response[i*RESPONSE_DATA_SIZE_U8+8] == 1 {
            positive_queries.push(query_id_from_u8(&response[i*RESPONSE_DATA_SIZE_U8..i*RESPONSE_DATA_SIZE_U8+8]));
        }
    }
    println!("[UNTRUSTED] positive result queryIds: {:?}", positive_queries);

    /* finish */
    enclave.destroy();
    // println!("[UNTRUSTED] All process is successful!!");
    clocker.show_all();
    if args[3] == "true".to_string() {
        let now: String = get_timestamp();
        write_to_file(
            format!("data/result/result-{}-finiteStateTranducer.txt", now),
            "simple hash and list".to_string(),
            c_filename.to_string(),
            q_filename.to_string(),
            threashould,
            "only risk_level".to_string(),
            clocker
        );
    }
}

fn encodedNoChunk() {
    let args = _get_options();
    /* parameters */
    let threashould: usize = args[0].parse().unwrap();
    let q_filename = &args[1];
    let c_filename = &args[2];

    let mut clocker = Clocker::new();

    /* read central data */
    clocker.set_and_start("Read Central Data");
    let external_data = EncodedData::read_raw_from_file(c_filename);
    clocker.stop("Read Central Data");

    /* preprocess central data */
    clocker.set_and_start("Distribute central data");
    let mut encoded_value_u8: Vec<u8> = Vec::with_capacity(100000000);
    let epoch_data_size = external_data.prepare_sgx_data(&mut encoded_value_u8);
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
    let query_data = EncodedQueryData::read_raw_from_file(q_filename);
    clocker.stop("Read Query Data");

    /* upload query data */
    clocker.set_and_start("ECALL upload_query_data");
    let mut retval = sgx_status_t::SGX_SUCCESS;
    let result = unsafe {
        upload_encoded_query_data(
            enclave.geteid(),
            &mut retval,
            query_data.total_data_to_u8().as_ptr() as * const u8,
            query_data.total_size(),
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
        private_encode_contact_trace(
            enclave.geteid(),
            &mut retval,
            encoded_value_u8.as_ptr() as * const u8,
            encoded_value_u8.len(),
            epoch_data_size
        )
    };
    match result {
        sgx_status_t::SGX_SUCCESS => {
            print!("\r[UNTRUSTED] private_contact_trace Succes! {} th iteration", 0);
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
        get_encoded_result(
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
    
    let mut positive_queries = vec![];
    for i in 0..query_data.client_size {
        if response[i*RESPONSE_DATA_SIZE_U8+8] == 1 {
            positive_queries.push(query_id_from_u8(&response[i*RESPONSE_DATA_SIZE_U8..i*RESPONSE_DATA_SIZE_U8+8]));
        }
    }
    println!("[UNTRUSTED] positive result queryIds: {:?}", positive_queries);

    /* finish */
    enclave.destroy();
    // println!("[UNTRUSTED] All process is successful!!");
    clocker.show_all();
    if args[3] == "true".to_string() {
        let now: String = get_timestamp();
        write_to_file(
            format!("data/result/result-{}-encodedNoChunk.txt", now),
            "simple hash and list".to_string(),
            c_filename.to_string(),
            q_filename.to_string(),
            threashould,
            "only risk_level".to_string(),
            clocker
        );
    }
}