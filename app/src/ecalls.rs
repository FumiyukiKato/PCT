extern crate sgx_types;
extern crate sgx_urts;
extern crate serde;
extern crate serde_json;

use sgx_types::*;
use sgx_urts::SgxEnclave;
use std::fs::File;
use std::io::BufReader;

static ENCLAVE_FILE: &'static str = "bin/enclave.signed.so";

extern {
    pub fn upload_query_data(eid: sgx_enclave_id_t, retval: *mut sgx_status_t,
                    total_query_data: * const u8, total_size: usize,
                    size_list: * const usize, client_size: usize, query_id_list: * const u64) -> sgx_status_t;
}

pub fn init_enclave() -> SgxResult<SgxEnclave> {
    let mut launch_token: sgx_launch_token_t = [0; 1024];
    let mut launch_token_updated: i32 = 0;
    // call sgx_create_enclave to initialize an enclave instance
    // Debug Support: set 2nd parameter to 1
    let debug = 1;
    let mut misc_attr = sgx_misc_attribute_t {secs_attr: sgx_attributes_t { flags:0, xfrm:0}, misc_select:0};
    SgxEnclave::create(ENCLAVE_FILE,
                       debug,
                       &mut launch_token,
                       &mut launch_token_updated,
                       &mut misc_attr)
}