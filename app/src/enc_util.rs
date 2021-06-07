use hex;
use util::*;

// バファリングするクエリはせいぜい10000なので64bitで余裕
pub type QueryId = u64;

pub fn encrypt_to_flat_vec_u8(query_data: &Vec<Vec<Vec<u8>>>, query_id_list: &Vec<u64>) -> Vec<u8> {
    let mut u8_vec_list: Vec<Vec<u8>> = Vec::with_capacity(query_data.len());
    query_data.iter().zip(query_id_list).for_each(|(detail, query_id)| {
        // encrypt by session key as secure channel to enclave.
        u8_vec_list.push(encypt_as_secure_channel_by_query_id(detail, *query_id));
    });
    let total_u8_vec: Vec<u8> = flatten(u8_vec_list);
    total_u8_vec
}

pub fn encrypt_central_data(central_byte_data: &Vec<u8>, key: u64) -> Vec<u8> {
    /* Remote Attestation Mock up */
    // Remote attestation is done and session (shared) key has been exchanged.
    // Here, suppose that shared key is simply derived from their query_id.

    let mut shared_key: [u8; 16] = [0; 16];
    shared_key[..8].copy_from_slice(&key.to_be_bytes());
    let counter_block: [u8; 16] = COUNTER_BLOCK;
    let ctr_inc_bits: u32 = SGXSSL_CTR_BITS;
    let src_len: usize = central_byte_data.len();
    let mut encrypted_buf: Vec<u8> = vec![0; src_len];

    let ret = unsafe { 
        sgx_aes_ctr_encrypt(
            &shared_key,
            central_byte_data.as_ptr() as * const u8,
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

fn encypt_as_secure_channel_by_query_id(detail: &Vec<Vec<u8>>, query_id: u64) -> Vec<u8> {
    /* Remote Attestation Mock up */
    // Remote attestation is done and session (shared) key has been exchanged.
    // Here, suppose that shared key is simply derived from their query_id.
   
    let mut u8_vec: Vec<u8> = flatten(detail.clone());
    let mut query_size = detail.len();

    let mut shared_key: [u8; 16] = [0; 16];
    shared_key[..8].copy_from_slice(&query_id.to_be_bytes());
    let counter_block: [u8; 16] = COUNTER_BLOCK;
    let ctr_inc_bits: u32 = SGXSSL_CTR_BITS;
    let src_len: usize = query_size*ENCODEDVALUE_SIZE;
    let mut encrypted_buf: Vec<u8> = vec![0; src_len];

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

    if ret < 0 {
        println!("Error in CTR encryption.");
        std::process::exit(-1);
    }
    encrypted_buf
}

fn flatten(nested: Vec<Vec<u8>>) -> Vec<u8> {
    nested.into_iter().flatten().collect()
}

fn hex_string_to_u8(hex_string: &String) -> Vec<u8> {
    let decoded = hex::decode(hex_string).expect("Decoding failed: Expect hex string!");
    decoded
}