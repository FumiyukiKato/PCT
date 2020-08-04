use serde::*;
use std::fs::File;
use std::io::BufReader;

pub const UNIXEPOCH_U8_SIZE: usize = 10;
pub const GEOHASH_U8_SIZE: usize = 10;
pub const QUERY_U8_SIZE: usize = UNIXEPOCH_U8_SIZE + GEOHASH_U8_SIZE;

pub const ENCODED_QUERY_SIZE: usize = 14;

// バファリングするクエリはせいぜい10000なので64bitで余裕
pub type QueryId = u64;


// query data sholud be no compressioned...
#[derive(Serialize, Deserialize, Debug)]
pub struct QueryData {
    pub data: Vec<QueryDataDetail>,
    pub client_size: usize,
}

impl QueryData { 
    pub fn read_raw_from_file(filename: &str) -> Self {
        let file = File::open(filename).unwrap();
        let reader = BufReader::new(file);
        let query_data: QueryData = serde_json::from_reader(reader).unwrap();
        if query_data.client_size != query_data.data.len() {
            println!("[Error] Invalid data format from {}!", filename);
            panic!()
        }
        query_data
    }

    pub fn total_size(&self) -> usize {
        let mut sum = 0;
        for data in self.data.iter() {
            sum += data.query_size*QUERY_U8_SIZE;
        }
        sum
    }

    pub fn total_data_to_u8(&self) -> Vec<u8> {
        let str_list: Vec<String> = self.data.iter().map(|detail| detail.geodata.clone()).collect();
        hex_string_to_u8(&str_list.join(""))
    }

    pub fn size_list(&self) -> Vec<usize> {
        self.data.iter().map(|d| d.query_size).collect()
    }

    pub fn query_id_list(&self) -> Vec<u64> {
        self.data.iter().map(|d| d.query_id).collect()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct QueryDataDetail {
    query_id: QueryId,
    geodata: String,
    query_size: usize,
}

fn hex_string_to_u8(hex_string: &String) -> Vec<u8> {
    let decoded = hex::decode(hex_string).expect("Decoding failed: Expect hex string!");
    decoded
}

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

    pub fn total_size(&self) -> usize {
        let mut sum = 0;
        for data in self.data.iter() {
            sum += data.query_size*ENCODED_QUERY_SIZE;
        }
        sum
    }

    pub fn total_data_to_u8(&self) -> Vec<u8> {
        let str_list: Vec<String> = self.data.iter().map(|detail| detail.geodata.clone()).collect();
        hex_string_to_u8(&str_list.join(""))
    }

    pub fn query_id_list(&self) -> Vec<u64> {
        self.data.iter().map(|d| d.query_id).collect()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EncodedQueryDataDetail {
    query_id: QueryId,
    geodata: String,
    query_size: usize,
}