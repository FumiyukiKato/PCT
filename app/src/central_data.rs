use serde::*;
use std::fs::File;
use std::io::BufReader;
use std::collections::HashMap;

/* 
ダックタイピングっぽくやりたいけどやり方が分からないので適当にやる 
*/
trait PCTDataStructure {
    fn from_ExternalDataJson(data: &ExternalDataJson) -> Self;
}

/* 
実装っぽい部分
*/
pub type PCTHash = HashMap<[u8; 10], Vec<u64>>;

impl PCTDataStructure for PCTHash {
    fn from_ExternalDataJson(data: &ExternalDataJson) -> Self {
        let mut hash = PCTHash::new();
        for v in data.vec.iter() {
            let mut geohash_u8 = [0_u8; 10];
            geohash_u8.copy_from_slice(hex_string_to_u8(&v.geohash).as_slice());
            match hash.get_mut(&geohash_u8) {
                Some(vec) => { vec.push(v.unixepoch) },
                None => { hash.insert(geohash_u8, vec![v.unixepoch]); },
            };
        }
        hash
    }
}

#[derive(Clone, Default, Debug)]
pub struct ExternalData<T> {
    data: T
}

impl<T> ExternalData<T> 
    where 
        T: PCTDataStructure {
    pub fn read_raw_from_file(filename: &str) -> ExternalData<T> {
        let file = File::open(filename).unwrap();
        let reader = BufReader::new(file);
        let data: ExternalDataJson = serde_json::from_reader(reader).unwrap();
        ExternalData { data: T::from_ExternalDataJson(&data) }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ExternalDataJson {
    vec: Vec<ExternalDataDetail>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ExternalDataDetail {
    geohash: String,
    unixepoch: u64,
}

fn hex_string_to_u8(hex_string: &String) -> Vec<u8> {
    let decoded = hex::decode(hex_string).expect("Decoding failed: Expect hex string!");
    decoded
}