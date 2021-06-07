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

use std::{env, fs::File, io::Write, path::Path};

extern crate cc;

fn main () {
    let out_dir = env::var("OUT_DIR").expect("No out dir");
    let dest_path = Path::new(&out_dir).join("init_constants.rs");
    let mut f = File::create(&dest_path).expect("Could not create file");

    let encoded_value_size = option_env!("ENCODEDVALUE_SIZE");
    let encoded_value_size: usize = encoded_value_size
        .expect("Could not parse MAX_DIMENSIONS")
        .parse()
        .expect("Could not parse MAX_DIMENSIONS");
    write!(&mut f, "pub const ENCODEDVALUE_SIZE: usize = {};", encoded_value_size)
        .expect("Could not write file");

    cc::Build::new()
        .cpp(true)
        .warnings(true)
        .file("src/cpp/encryption.cpp")
        .cpp_link_stdlib("crypto")
        .compile("encryption");

    let sdk_dir = env::var("SGX_SDK")
                    .unwrap_or_else(|_| "/opt/intel/sgxsdk".to_string());
    let is_sim = env::var("SGX_MODE")
                    .unwrap_or_else(|_| "HW".to_string());

    println!("cargo:rustc-link-search=native=../lib");
    println!("cargo:rustc-link-lib=static=Enclave_u");

    println!("cargo:rustc-link-search=native={}/lib64", sdk_dir);
    match is_sim.as_ref() {
        "SW" => println!("cargo:rustc-link-lib=dylib=sgx_urts_sim"),
        "HW" => println!("cargo:rustc-link-lib=dylib=sgx_urts"),
        _    => println!("cargo:rustc-link-lib=dylib=sgx_urts"), // Treat undefined as HW
    }
}
