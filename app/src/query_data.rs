use serde::*;
use std::fs::File;
use std::io::BufReader;
use hex;
use util::*;

// バファリングするクエリはせいぜい10000なので64bitで余裕
pub type QueryId = u64;

// query data sholud be no compressioned...
#[derive(Serialize, Deserialize, Debug)]
pub struct EncodedQueryData {
    pub data: Vec<EncodedQueryDataDetail>,
    pub client_size: usize,
}

impl EncodedQueryData { 
    pub fn read_raw_from_file(filename: &str) -> Self {
        let file = File::open(filename).unwrap();
        let reader = BufReader::new(file);
        let query_data: EncodedQueryData = serde_json::from_reader(reader).unwrap();
        if query_data.client_size != query_data.data.len() {
            println!("[Error] Invalid data format from {}!", filename);
            panic!()
        }
        query_data
    }

    pub fn total_data_to_u8(&self) -> Vec<u8> {
        let mut u8_vec_list: Vec<Vec<u8>> = Vec::with_capacity(self.client_size);
        self.data.iter().for_each(|detail| {
            // encrypt by session key as secure channel to enclave.
            u8_vec_list.push(encryptAsSecureChannel(detail));
        });
        let total_u8_vec: Vec<u8> = flatten(u8_vec_list);
        total_u8_vec
    }

    pub fn query_id_list(&self) -> Vec<u64> {
        self.data.iter().map(|d| d.query_id).collect()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EncodedQueryDataDetail {
    pub query_id: QueryId,
    pub geodata: Vec<String>,
    pub query_size: usize,
}

fn encryptAsSecureChannel(detail: &EncodedQueryDataDetail) -> Vec<u8> {
    /* Remote Attestation Mock up */
    // Remote attestation is done and session (shared) key has been exchanged.
    // Here, suppose that shared key is simply derived from their query_id.
   
    #[cfg(any(feature = "th72", feature = "th48", feature = "th54", feature = "th60"))]
    let mut u8_vec: Vec<u8> = Vec::with_capacity(detail.query_size);
    #[cfg(any(feature = "th72", feature = "th48", feature = "th54", feature = "th60"))]
    for base8 in detail.geodata.iter() {
        u8_vec.extend_from_slice(base8decode(base8.to_string()).as_slice());
    }

    let mut shared_key: [u8; 16] = [0; 16];
    shared_key[..8].copy_from_slice(&detail.query_id.to_be_bytes());
    let counter_block: [u8; 16] = COUNTER_BLOCK;
    let ctr_inc_bits: u32 = SGXSSL_CTR_BITS;
    let src_len: usize = detail.query_size*ENCODEDVALUE_SIZE;
    let mut encrypted_buf: Vec<u8> = vec![0; src_len];

    #[cfg(any(feature = "th72", feature = "th48", feature = "th54", feature = "th60"))]
    let ret = unsafe { 
        sgx_aes_ctr_encrypt(
            &shared_key,
            u8_vec.as_ptr() as * const u8,
            src_len as u32,
            &counter_block as * const u8,
            ctr_inc_bits,
            encrypted_buf.as_mut_ptr()
        )
    };

    #[cfg(any(feature = "gp10"))]
    let ret = unsafe { 
        sgx_aes_ctr_encrypt(
            &shared_key,
            detail.geodata.join("").as_bytes().as_ptr() as * const u8,
            src_len as u32,
            &counter_block as * const u8,
            ctr_inc_bits,
            encrypted_buf.as_mut_ptr()
        )
    };


    if ret < 0 {
        println!("Error in CTR encryption.");
        std::process::exit(-1);
    }
    encrypted_buf
}

fn flatten<T>(nested: Vec<Vec<T>>) -> Vec<T> {
    nested.into_iter().flatten().collect()
}

fn hex_string_to_u8(hex_string: &String) -> Vec<u8> {
    let decoded = hex::decode(hex_string).expect("Decoding failed: Expect hex string!");
    decoded
}