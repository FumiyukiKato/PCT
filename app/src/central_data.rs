use serde::*;
use std::fs::File;
use std::io::BufReader;
use std::collections::HashMap;
use std::vec::Vec;
use std::collections::HashSet;
use fst::{Set};
use bincode;
use util::*;


/* Type Period */
pub type UnixEpoch = u64;
// UNIX EPOCH INTERVAL OF THE GPS DATA
pub const TIME_INTERVAL: u64 = 600;

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

    pub fn size(&self) -> usize {
        self.structure.len()
    }

    pub fn read_raw_from_file(filename: &str) -> Self {
        let file = File::open(filename).unwrap();
        let reader = BufReader::new(file);
        let data: ExternalEncodedDataJson = serde_json::from_reader(reader).unwrap();
        
        let mut set: HashSet<EncodedValue> = HashSet::with_capacity(100000);
        if cfg!(any(feature = "th64", feature = "th48", feature = "th42", feature = "th36")) {
            for v in data.data.iter() {
                let mut encoded_value_u8: EncodedValue = [0_u8; ENCODEDVALUE_SIZE];
                encoded_value_u8.copy_from_slice(base8decode(v.to_string()).as_slice());
                set.insert(encoded_value_u8);
            }
        } else if cfg!(feature = "gp10") {
            for v in data.data.iter() {
                let mut encoded_value_u8: EncodedValue = [0_u8; ENCODEDVALUE_SIZE];
                // ascii-code
                encoded_value_u8.copy_from_slice(v.as_bytes());
                set.insert(encoded_value_u8);
            }    
        } else {
            panic!("Error attribute 'encode' is wrong.")
        }

        let vec: Vec<EncodedValue> = set.into_iter().collect();
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

#[derive(Clone, Default, Debug)]
pub struct CentralFST {
    data: Vec<Vec<u8>>,
}

impl CentralFST {
    pub fn new() -> Self {
        CentralFST {
            data: Vec::with_capacity(100),
        }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn prepare_sgx_data(&self, index: usize) -> &Vec<u8> {
        &self.data[index]
    }

    pub fn from_EncodedData(encoded_data: EncodedData, threashould: usize) -> Self {
        let mut encoded_value_vec = encoded_data.structure;
        encoded_value_vec.sort();

        let mut ordered_vec: Vec<EncodedValue> = vec![];
        let mut this = CentralFST::new();

        for (i, value) in encoded_value_vec.iter().enumerate() {
            ordered_vec.push(*value);
            if (i+1) % threashould == 0 {
                let bytes: Vec<u8> = Set::from_iter(ordered_vec)
                    .unwrap().as_ref().as_bytes().to_vec();
                println!(" r_i (server side chunk data) size = {} bytes", bytes.len());
                this.data.push(bytes);
                ordered_vec = vec![];
            }
        }
        if ordered_vec.len() > 0 {
            let bytes = Set::from_iter(ordered_vec)
                .unwrap().as_ref().as_bytes().to_vec();
            println!(" r_i (server side chunk data) size = {} bytes", bytes.len());
            this.data.push(bytes);
        }
        this
    }
}

#[derive(Clone, Default, Debug)]
pub struct CentralHashSet {
    data: Vec<Vec<u8>>,
}

impl CentralHashSet {
    pub fn new() -> Self {
        CentralHashSet {
            data: Vec::with_capacity(100),
        }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn prepare_sgx_data(&self, index: usize) -> &Vec<u8> {
        &self.data[index]
    }

    pub fn from_EncodedData(encoded_data: EncodedData, threashould: usize) -> Self {
        let mut encoded_value_vec = encoded_data.structure;
        encoded_value_vec.sort();

        let mut hashset: HashSet<EncodedValue> = HashSet::with_capacity(threashould);
        
        let mut this = CentralHashSet::new();
        for (i, value) in encoded_value_vec.iter().enumerate() {
            hashset.insert(*value);
            if (i+1) % threashould == 0 {
                let bytes: Vec<u8> = bincode::serialize(&hashset).unwrap();
                println!("[HashSet] r_i size = {} bytes", bytes.len());
                this.data.push(bytes);
                hashset = HashSet::with_capacity(threashould);
            }
        }
        if hashset.len() > 0 {
            let bytes: Vec<u8> = bincode::serialize(&hashset).unwrap();
            println!("[HashSet] r_i size = {} bytes", bytes.len());
            this.data.push(bytes);
        }
        this
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