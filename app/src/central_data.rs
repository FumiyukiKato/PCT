use std::vec::Vec;
use std::collections::HashSet;
use succinct_trie::trie::Trie;
use bincode;
use std::mem;
use util::ENCODEDVALUE_SIZE;

use crate::enc_util::encrypt_central_data;


/* rie
    チャンク化しない
*/
pub type EncodedValue = Vec<u8>;
const CENTRAL_KEY: u64 = 777;

// vector of binary central data 
#[derive(Clone, Default, Debug)]
pub struct CentralTrie {
    data: Vec<Vec<u8>>,
}

impl CentralTrie {
    pub fn new() -> Self {
        CentralTrie {
            data: Vec::with_capacity(100),
        }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn prepare_sgx_data(&self, index: usize) -> &Vec<u8> {
        &self.data[index]
    }

    pub fn from_encoded_data(mut encoded_data: Vec<Vec<u8>>, threashould: usize) -> Self {
        encoded_data.sort();

        let mut ordered_vec: Vec<EncodedValue> = vec![];
        let mut this = CentralTrie::new();

        for (i, value) in encoded_data.iter().enumerate() {
            ordered_vec.push(value.clone());
            if (i+1) % threashould == 0 {
                let trie: Trie = Trie::new(&ordered_vec);
                println!(" r_i (server side chunk data) size = {} bytes", trie.byte_size());
                let bytes = trie.serialize();
                this.data.push(encrypt_central_data(&bytes, CENTRAL_KEY));
                ordered_vec = vec![];
            }
        }
        if ordered_vec.len() > 0 {
            let trie: Trie = Trie::new(&ordered_vec);
            println!(" r_i (server side chunk data) size = {} bytes", trie.byte_size());
            let bytes = trie.serialize();
            println!("Trie byte len = {}", bytes.len());
            this.data.push(encrypt_central_data(&bytes, CENTRAL_KEY));
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

    pub fn from_encoded_data(mut encoded_data: Vec<Vec<u8>>, threashould: usize) -> Self {
        encoded_data.sort();

        let mut hashset: HashSet<EncodedValue> = HashSet::with_capacity(0);
        
        let mut this = CentralHashSet::new();
        for (i, value) in encoded_data.iter().enumerate() {
            hashset.insert(value.clone());
            if (i+1) % threashould == 0 {
                let bytes: Vec<u8> = bincode::serialize(&hashset).unwrap();
                println!("[HashSet] r_i size = {} bytes", bytes.len());
                this.data.push(bytes);
                hashset = HashSet::with_capacity(threashould);
            }
        }
        if hashset.len() > 0 {
            println!("[HashSet] len {}", hashset.len());
            let bytes: Vec<u8> = bincode::serialize(&hashset).unwrap();
            println!("[HashSet] r_i size = {} bytes", bytes.len());
            println!("HashTable size = {} bytes", (hashset.capacity() * 11 / 10) * (ENCODEDVALUE_SIZE + mem::size_of::<u64>()));
            this.data.push(bytes);
        }
        this
    }
}

#[derive(Clone, Default, Debug)]
pub struct NonPrivateHashSet {
    pub set: HashSet<EncodedValue>,
}

impl NonPrivateHashSet {
    pub fn new() -> Self {
        NonPrivateHashSet {
            set: HashSet::default(),
        }
    }

    pub fn from_encoded_data(encoded_data: Vec<Vec<u8>>) -> Self {
        let mut this = NonPrivateHashSet::new();

        for value in encoded_data {
            this.set.insert(value);
        }
        this
    }

    pub fn calc_memory(&self) {
        println!("HashTable size = {} bytes", (self.set.capacity() * 11 / 10) * (mem::size_of::<EncodedValue>() + mem::size_of::<()>() + mem::size_of::<u64>()));
    }
}

pub struct NonPrivateFSA {
    pub set: Trie,
}

impl NonPrivateFSA {
    pub fn new() -> Self {
        NonPrivateFSA {
            set: Trie::new(&vec![vec![]])
        }
    }

    pub fn from_encoded_data(mut encoded_data: Vec<Vec<u8>>) -> Self {
        encoded_data.sort();
        let mut this = NonPrivateFSA::new();
        this.set = Trie::new(&encoded_data);
        this
    }

    pub fn calc_memory(&self) {
        println!("FSA size = {} bytes", self.set.byte_size());
    }
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