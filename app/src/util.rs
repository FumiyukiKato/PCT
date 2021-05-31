use glob::glob;
use sgx_types::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::time::{Duration, Instant};
use std::{
    char,
    io::{BufRead, BufReader},
};
use regex::Regex;

pub const ENCODEDVALUE_SIZE: usize = 8;

#[cfg(feature = "gp10")]
pub const ENCODEDVALUE_SIZE: usize = 14;

#[cfg(feature = "th64")]
pub const ENCODEDVALUE_SIZE: usize = 8;

#[cfg(feature = "th56")]
pub const ENCODEDVALUE_SIZE: usize = 7;

pub const SGXSSL_CTR_BITS: u32 = 128;
pub const COUNTER_BLOCK: [u8; 16] = [0; 16];

pub type sgx_aes_ctr_128bit_key_t = [uint8_t; 16];
extern "C" {
    pub fn sgx_aes_ctr_decrypt(
        p_key: *const sgx_aes_ctr_128bit_key_t,
        p_src: *const uint8_t,
        src_len: uint32_t,
        p_ctr: *const uint8_t,
        ctr_inc_bits: uint32_t,
        p_dst: *mut uint8_t,
    ) -> u32;

    pub fn sgx_aes_ctr_encrypt(
        p_key: *const sgx_aes_ctr_128bit_key_t,
        p_src: *const uint8_t,
        src_len: uint32_t,
        p_ctr: *const uint8_t,
        ctr_inc_bits: uint32_t,
        p_dst: *mut uint8_t,
    ) -> u32;
}

#[derive(Clone, Default, Debug)]
pub struct Clocker<'a> {
    data: HashMap<&'a str, Instant>,
    result: HashMap<&'a str, Duration>,
}

impl<'a> Clocker<'a> {
    pub fn new() -> Self {
        Clocker::default()
    }

    pub fn set_and_start(&mut self, name: &'a str) {
        // println!("[Clocker] {} start.", name);
        self.data.insert(name, Instant::now());
    }

    pub fn stop(&mut self, name: &'a str) {
        match self.data.get_mut(name) {
            Some(instant) => {
                let duration = instant.elapsed();
                self.result.insert(name, duration);
                // println!("[Clocker] {} end.", name);
            }
            None => {
                println!("[Clocker] error!! {} is not found", name);
            }
        }
    }

    pub fn show_all(&self) {
        for (name, duration) in self.result.iter() {
            println!(
                "[Clocker] {}:  {}.{:06} seconds",
                name,
                duration.as_secs(),
                duration.subsec_nanos() / 1_000
            );
        }
    }

    pub fn to_string(&self) -> String {
        let mut res = String::new();
        for (name, duration) in self.result.iter() {
            res = format!(
                "{}{:<30}:  {}.{:06} seconds\n",
                res,
                name,
                duration.as_secs(),
                duration.subsec_nanos() / 1_000
            );
        }
        res
    }
}

pub fn read_trajectory_hash_from_csv(filename: &str) -> Vec<Vec<u8>> {
    let file = File::open(filename).expect("file open error");
    let reader = BufReader::new(file);
    let mut hash_vec = Vec::new();
    for line in reader.lines().into_iter() {
        if let Ok(hash) = line {
            let chars: Vec<char> = hash.chars().collect();
            let mut hash_bytes: Vec<u8> = Vec::with_capacity(hash.len() / 2);
            for i in 0..(hash.len() / 2) {
                hash_bytes.push(16 * hex_to_num(chars[2 * i]) + hex_to_num(chars[2 * i + 1]));
            }
            hash_vec.push(hash_bytes);
        }
    }
    hash_vec
}

pub fn read_trajectory_hash_from_csv_for_clients(dirname: &str) -> Vec<Vec<Vec<u8>>> {
    let mut query_data = Vec::new();
    let re = Regex::new(r".*/client-(?P<client_id>\d+).*.csv").unwrap();
    for entry in glob(format!("{}/*.csv", dirname).as_str()).expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                let filepath = path.to_str().unwrap();
                let caps = match re.captures(filepath) {
                    Some(c) => c,
                    None => continue,
                };
                let client_id: u32 = caps["client_id"].parse().unwrap();

                println!("start ... filepath {}, client_id {}", filepath, client_id);
                let client_data = read_trajectory_hash_from_csv(path.to_str().unwrap());

                query_data.push(client_data);
            }
            Err(_) => panic!("failed to find path"),
        }
    }
    query_data
}

fn hex_to_num(c: char) -> u8 {
    match c {
        '0' => 0,
        '1' => 1,
        '2' => 2,
        '3' => 3,
        '4' => 4,
        '5' => 5,
        '6' => 6,
        '7' => 7,
        '8' => 8,
        '9' => 9,
        'a' => 10,
        'b' => 11,
        'c' => 12,
        'd' => 13,
        'e' => 14,
        'f' => 15,
        _ => panic!("invalid hex string"),
    }
}

pub fn write_to_file(
    file_name: String,
    data_structure_type: String,
    method: String,
    central_data_file: String,
    central_data_size: usize,
    query_data_file: String,
    client_size: usize,
    query_size: usize,
    threashould: usize,
    clocker: Clocker,
) {
    let mut file = File::create(file_name).unwrap();
    let clocker_result: String = format!(
        r#"
+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
Basic data
+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
data structure type       : {data_structure_type}
encoding method           : {method}
threashould               : {threashould}
central data file         : size = {central_data_size}, {central_data_file}
query data file           : size = {query_size} x {client_size}, {query_data_file}
+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
Clocker data
+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
{clocker_string}
-----------------------------------------------------------------------------
"#,
        data_structure_type = data_structure_type,
        method = method,
        central_data_file = central_data_file,
        central_data_size = central_data_size,
        query_data_file = query_data_file,
        query_size = query_size,
        client_size = client_size,
        threashould = threashould,
        clocker_string = clocker.to_string()
    );

    file.write_all(clocker_result.as_bytes()).unwrap();
}

pub fn get_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => format!("{}", n.as_secs()),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    }
}

pub fn query_id_from_u8(query_id: &[u8]) -> u64 {
    let mut array: [u8; 8] = [0; 8];
    array.copy_from_slice(query_id);
    u64::from_be_bytes(array)
}

pub fn base8decode(base8: String) -> Vec<u8> {
    let mut bitvec: Vec<u8> = vec![];
    let mut vec: Vec<u8> = vec![];
    for i in base8.as_bytes().iter() {
        bitvec.extend(base8map(*i));
    }
    // padding
    if bitvec.len() % 8 > 0 {
        for i in 0..(8 - bitvec.len() % 8) {
            bitvec.push(0b0)
        }
    }
    assert_eq!(bitvec.len() % 8, 0);
    for i in 0..(bitvec.len() / 8) {
        vec.push(bitVecToByte(&bitvec[i * 8..(i + 1) * 8]));
    }
    vec
}

fn bitVecToByte(bitvec: &[u8]) -> u8 {
    assert_eq!(bitvec.len() % 8, 0);
    let byte: u8 = bitvec[7] * 128
        + bitvec[6] * 64
        + bitvec[5] * 32
        + bitvec[4] * 16
        + bitvec[3] * 8
        + bitvec[2] * 4
        + bitvec[1] * 2
        + bitvec[0] * 1;
    byte
}

fn base8map(byte: u8) -> Vec<u8> {
    return match byte {
        48 => vec![0b0, 0b0, 0b0], // 0
        49 => vec![0b0, 0b0, 0b1], // 1
        50 => vec![0b0, 0b1, 0b0], // 2
        51 => vec![0b0, 0b1, 0b1], // 3
        52 => vec![0b1, 0b0, 0b0], // 4
        53 => vec![0b1, 0b0, 0b1], // 5
        54 => vec![0b1, 0b1, 0b0], // 6
        55 => vec![0b1, 0b1, 0b1], // 7
        _ => panic!("decode error!"),
    };
}
