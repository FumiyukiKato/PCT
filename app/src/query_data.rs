use serde::*;
use std::fs::File;
use std::io::BufReader;
use hex;

pub const ENCODED_QUERY_SIZE: usize = 14;

// バファリングするクエリはせいぜい10000なので64bitで余裕
pub type QueryId = u64;

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
        let mut str_list: Vec<String> = Vec::with_capacity(self.client_size*1000);
        self.data.iter().for_each(|detail| {
            for v in detail.geodata.iter() {
                str_list.push(v.clone());
            }
        });
        return Vec::<u8>::from(str_list.join("").as_bytes());
    }

    pub fn query_id_list(&self) -> Vec<u64> {
        self.data.iter().map(|d| d.query_id).collect()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EncodedQueryDataDetail {
    query_id: QueryId,
    geodata: Vec<String>,
    query_size: usize,
}