use serde::*;

const UNIXEPOCH_U8_SIZE: usize = 10;
const GEOHASH_U8_SIZE: usize = 10;
const QUERY_U8_SIZE: usize = UNIXEPOCH_U8_SIZE + GEOHASH_U8_SIZE;

// バファリングするクエリはせいぜい10000なので64bitで余裕
pub type QueryId = u64;

#[derive(Serialize, Deserialize, Debug)]
pub struct QueryData {
    pub data: Vec<QueryDataDetail>,
    pub client_size: usize,
}

impl QueryData { 
    pub fn total_size(&self) -> usize {
        let mut sum = 0;
        for data in self.data.iter() {
            sum += data.query_size*QUERY_U8_SIZE;
        }
        sum
    }

    pub fn total_data_to_u8(&self) -> Vec<u8> {
        let str_list: Vec<String> = self.data.iter().map(|detail| detail.geodata.clone()).collect();
        hex_string_to_u8(str_list.join(""))
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

fn hex_string_to_u8(hex_string: String) -> Vec<u8> {
    let decoded = hex::decode(hex_string).expect("Decoding failed: Expect hex string!");
    decoded
}