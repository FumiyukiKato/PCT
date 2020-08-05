use serde::*;
use std::fs::File;
use std::io::BufReader;
use std::collections::HashMap;
use std::vec::Vec;
use std::collections::HashSet;
use fst::{Set};


/* Type Period */
pub type UnixEpoch = u64;
// UNIX EPOCH INTERVAL OF THE GPS DATA
pub const TIME_INTERVAL: u64 = 600;
const GEOHASH_U8_SIZE: usize = 10;
const TIMEHASH_U8_SIZE: usize = 4;
const ENCODEDVALUE_SIZE: usize = GEOHASH_U8_SIZE + TIMEHASH_U8_SIZE;

/* GeohashTableWithPeriodArray */
#[derive(Clone, Default, Debug)]
pub struct GeohashTableWithPeriodArray {
    structure: HashMap<[u8; GEOHASH_U8_SIZE], Vec<Period>>
}

impl GeohashTableWithPeriodArray {
    pub fn new() -> Self {
        GeohashTableWithPeriodArray {
            structure: HashMap::with_capacity(10000)
        }
    }

    pub fn size(&self) -> usize {
        self.structure.len()
    }

    pub fn read_raw_from_file(filename: &str) -> Self {
        let external_data = GeohashTable::read_raw_from_file(filename);
        Self::geohash_based_compress(&external_data)
    }
    
    fn geohash_based_compress(original_data: &GeohashTable) -> GeohashTableWithPeriodArray {
        let mut geohash_table = GeohashTableWithPeriodArray::new();
        for (geohash, unixepoch_vec) in original_data.structure.iter() {
            geohash_table.structure.insert(*geohash, Period::from_unixepoch_vector(unixepoch_vec));
        }
        geohash_table
    }

    // Period側の合計データ数がthreashould以下になるようにチャンクに分ける
    pub fn disribute(&self, buf: &mut Vec<Self>, threashould: usize) {
        let mut val_num = 0;
        let mut data = Self::new();
        for (key, val) in self.structure.iter() {
            val_num += val.len();
            if val_num > threashould {
                val_num = val.len();
                buf.push(data);
                data = Self::new();
            }
            data.structure.insert(*key, val.to_vec());
        }
        if data.size() > 0 {
            buf.push(data);
        }
    }

    pub fn prepare_sgx_data(&self, geohash_u8: &mut Vec<u8>, period_u64: &mut Vec<u64>, size_list: &mut Vec<usize>) -> usize {
        let mut i = 0;
        for (key, value) in self.structure.iter() {
            let length = value.len();
            size_list.push(length);
            geohash_u8.extend_from_slice(key);
            let mut ret_vec: Vec<u64> = Vec::with_capacity(value.len() * 2);
            for period in value.iter() {
                ret_vec.push(period.0);
                ret_vec.push(period.1);
            }
            period_u64.extend_from_slice(ret_vec.as_slice());
            i += 1;
        }
        i
    }
}


/* GeohashTable 
    単純なハッシュマップ
    キーがgeohash, バリューがUnix epochのベクタ
*/
#[derive(Clone, Default, Debug)]
pub struct GeohashTable {
    structure: HashMap<[u8; GEOHASH_U8_SIZE], Vec<u64>>
}

impl GeohashTable {
    pub fn new() -> Self {
        GeohashTable {
            structure: HashMap::with_capacity(10000)
        }
    }

    pub fn size(&self) -> usize {
        self.structure.len()
    }

    pub fn read_raw_from_file(filename: &str) -> Self {
        let file = File::open(filename).unwrap();
        let reader = BufReader::new(file);
        let data: ExternalDataJson = serde_json::from_reader(reader).unwrap();
        
        let mut hash = GeohashTable::new();
        for v in data.vec.iter() {
            let mut geohash_u8 = [0_u8; GEOHASH_U8_SIZE];
            geohash_u8.copy_from_slice(hex_string_to_u8(&v.geohash).as_slice());
            match hash.structure.get_mut(&geohash_u8) {
                // centralデータに関しては，こいつがunique soted listである責務を持つ
                Some(sorted_list) => { _sorted_push(sorted_list, v.unixepoch) },
                None => { hash.structure.insert(geohash_u8, vec![v.unixepoch]); },
            };
        }
        println!("[UNTRUSTED] central data size {}", hash.size());
        hash
    }
    
    // Unixepoch側の合計データ数がthreashould以下になるようにチャンクに分ける
    // オペレーション的には，バッチ的にチャンク化しておくのが良さそう
    pub fn disribute(&self, buf: &mut Vec<Self>, threashould: usize) {
        let mut val_num = 0;
        let mut data = Self::new();
        for (key, val) in self.structure.iter() {
            val_num += val.len();
            if val_num > threashould {
                val_num = val.len();
                buf.push(data);
                data = Self::new();
            }
            data.structure.insert(*key, val.to_vec());
        }
        if data.size() > 0 {
            buf.push(data);
        }
    }

    // データは geohash, [u8], geohash, [u8],... と [u8]のサイズの配列というフォーマットでシリアライスする
    // 時間がかかっていそうならシリアライズは先にまとめて計算しておく
    // extend_from_sliceを使ったやり方(pushとかじゃなくてコピーするようにすれば少しだけ早くなる余地がある？)
    pub fn prepare_sgx_data(&self, geohash_u8: &mut Vec<u8>, unixepoch_u64: &mut Vec<u64>, size_list: &mut Vec<usize>) -> usize {
        let mut i = 0;
        for (key, value) in self.structure.iter() {
            let length = value.len();
            size_list.push(length);
            geohash_u8.extend_from_slice(key);
            unixepoch_u64.extend_from_slice(value);
            i += 1;
        }
        i
    }
}

/* PlainTable
    チャンク化しない
*/
#[derive(Clone, Default, Debug)]
pub struct PlainTable {
    structure: HashMap<[u8; GEOHASH_U8_SIZE], Vec<u64>>
}

impl PlainTable {
    pub fn new() -> Self {
        PlainTable {
            structure: HashMap::with_capacity(10000)
        }
    }

    pub fn size(&self) -> usize {
        self.structure.len()
    }

    pub fn read_raw_from_file(filename: &str) -> Self {
        let file = File::open(filename).unwrap();
        let reader = BufReader::new(file);
        let data: ExternalDataJson = serde_json::from_reader(reader).unwrap();
        
        let mut hash = PlainTable::new();
        for v in data.vec.iter() {
            let mut geohash_u8 = [0_u8; GEOHASH_U8_SIZE];
            geohash_u8.copy_from_slice(hex_string_to_u8(&v.geohash).as_slice());
            match hash.structure.get_mut(&geohash_u8) {
                // centralデータに関しては，こいつがunique soted listである責務を持つ
                Some(sorted_list) => { _sorted_push(sorted_list, v.unixepoch) },
                None => { hash.structure.insert(geohash_u8, vec![v.unixepoch]); },
            };
        }
        println!("[UNTRUSTED] central data size {}", hash.size());
        hash
    }

    // データは geohash, [u8], geohash, [u8],... と [u8]のサイズの配列というフォーマットでシリアライスする
    // 時間がかかっていそうならシリアライズは先にまとめて計算しておく
    // extend_from_sliceを使ったやり方(pushとかじゃなくてコピーするようにすれば少しだけ早くなる余地がある？)
    pub fn prepare_sgx_data(&self, geohash_u8: &mut Vec<u8>, unixepoch_u64: &mut Vec<u64>, size_list: &mut Vec<usize>) -> usize {
        let mut i = 0;
        for (key, value) in self.structure.iter() {
            let length = value.len();
            size_list.push(length);
            geohash_u8.extend_from_slice(key);
            unixepoch_u64.extend_from_slice(value);
            i += 1;
        }
        i
    }
}

/* TrajectoryTrie
    チャンク化しない
*/
type EncodedValue = [u8; ENCODEDVALUE_SIZE];

#[derive(Clone, Default, Debug)]
pub struct EncodedData {
    structure: Vec<EncodedValue>
}

impl EncodedData {
    pub fn new() -> Self {
        EncodedData {
            structure: Vec::with_capacity(1000000)
        }
    }

    pub fn read_raw_from_file(filename: &str) -> Self {
        let file = File::open(filename).unwrap();
        let reader = BufReader::new(file);
        let data: ExternalEncodedDataJson = serde_json::from_reader(reader).unwrap();
        
        let mut set: HashSet<EncodedValue> = HashSet::with_capacity(100000);
        for v in data.data.iter() {
            let mut encoded_value_u8: EncodedValue = [0_u8; ENCODEDVALUE_SIZE];
            encoded_value_u8.copy_from_slice(v.as_bytes());
            set.insert(encoded_value_u8);
        }
        println!("[UNTRUSTED] unique data size {}", set.len());
        let mut vec: Vec<EncodedValue> = set.into_iter().collect();
        vec.sort();
        let mut set = Set::from_iter(&vec).unwrap();
        println!("[FST] fst R size {} bytes", set.as_ref().size());
        EncodedData { structure: vec }
    }
    
    pub fn prepare_sgx_data(&self, encoded_value_u8: &mut Vec<u8>) -> usize {
        let mut i = 0;
        for value in self.structure.iter() {
            encoded_value_u8.extend_from_slice(value);
            i += 1;
        }
        i
    }

    pub fn disribute(&self, buf: &mut Vec<Self>, threashould: usize) {
        let mut val_num = 0;
        let mut data = Self::new();
        for (i, value) in self.structure.iter().enumerate() {
            data.structure.push(*value);
            if (i+1) % threashould == 0 {
                buf.push(data);
                data = Self::new();
            }
        }
        if data.structure.len() > 0 {
            buf.push(data);
        }
    }
}

/* 補助的なものたち */

#[derive(Clone, Default, Debug)]
pub struct Period(UnixEpoch, UnixEpoch);

impl Period {
    pub fn new() -> Self {
        Period::default()
    }

    pub fn with_start(start: UnixEpoch) -> Self {
        Period(start, start)
    }

    pub fn from_unixepoch_vector(unixepoch_vec: &Vec<UnixEpoch>) -> Vec<Period> {
        let mut period_vec: Vec<Period> = vec![];
        
        assert!(unixepoch_vec.len() > 0);
        let mut latest_unixepoch: UnixEpoch = unixepoch_vec[0];
        let mut period = Period::with_start(latest_unixepoch);
        
        for unixepoch in unixepoch_vec.iter() {
            if latest_unixepoch + TIME_INTERVAL >= *unixepoch {
                latest_unixepoch = *unixepoch;
            } else {
                period.1 = latest_unixepoch;
                period_vec.push(period);
                period = Period::with_start(*unixepoch);
                latest_unixepoch = *unixepoch;
            }
        }
        period.1 = latest_unixepoch;
        period_vec.push(period);
        period_vec
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ExternalEncodedDataJson {
    data: Vec<String>,
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

// 昇順ソート+ユニーク性，O(n^2)だけどサイズは小さいので気にしない
// あえてジェネリクスにする必要はない，むしろ型で守っていく
fn _sorted_push(sorted_list: &mut Vec<u64>, unixepoch: u64) {
    let mut index = 0;
    for elm in sorted_list.iter() {
        if *elm > unixepoch {
            sorted_list.insert(index, unixepoch);
            return;
        } else if *elm == unixepoch {
            return;
        } else {
            index += 1;
        }
    }
    sorted_list.push(unixepoch);
}