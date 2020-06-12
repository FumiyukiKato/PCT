use serde::*;
use std::fs::File;
use std::io::BufReader;
use std::collections::HashMap;
use std::vec::Vec;

/* 
実装っぽい部分
*/

/*
単純なハッシュマップ
    キーがgeohash, バリューがUnix epochのベクタ
*/
#[derive(Clone, Default, Debug)]
pub struct PCTHash {
    structure: HashMap<[u8; 10], Vec<u64>>
}

impl PCTHash {
    pub fn new() -> Self {
        PCTHash::default()
    }

    pub fn size(&self) -> usize {
        self.structure.len()
    }

    pub fn read_raw_from_file(filename: &str) -> Self {
        let file = File::open(filename).unwrap();
        let reader = BufReader::new(file);
        let data: ExternalDataJson = serde_json::from_reader(reader).unwrap();
        
        let mut hash = PCTHash::new();
        for v in data.vec.iter() {
            let mut geohash_u8 = [0_u8; 10];
            geohash_u8.copy_from_slice(hex_string_to_u8(&v.geohash).as_slice());
            match hash.structure.get_mut(&geohash_u8) {
                Some(vec) => { vec.push(v.unixepoch) },
                None => { hash.structure.insert(geohash_u8, vec![v.unixepoch]); },
            };
        }
        hash
    }
    
    // Unixepoch側の合計データ数がthreashould以下になるようにチャンクに分ける
    // オペレーション的には，バッチ的にチャンク化しておくのが良さそう
    pub fn disribute(&self, buf: &mut Vec<PCTHash>, threashould: usize) {
        let mut val_num = 0;
        let mut data = PCTHash::new();
        for (key, val) in self.structure.iter() {
            val_num += val.len();
            if val_num > threashould {
                val_num = val.len();
                buf.push(data);
                data = PCTHash::new();
            }
            data.structure.insert(*key, val.to_vec());
        }
        if (data.size() > 0) {
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