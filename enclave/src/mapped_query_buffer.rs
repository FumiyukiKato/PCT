use std::collections::HashMap;
use std::vec::Vec;

use primitive::*;
use query_buffer::QueryBuffer;
use utils::*;


/* 
Type MappedQueryBuffer "Q"
    こっちのクエリ側のデータ構造も変わる可能性がある
    いい感じに抽象化するのがめんどくさいのでこのデータ構造自体を変える
    map.vec<Unixepoch>がソート済みでユニークセットになっていることは呼び出し側が保証している
*/
#[derive(Clone, Default, Debug)]
pub struct MappedQueryBuffer {
    pub map: Vec<(GeoHashKey, Vec<UnixEpoch>)>,
    // pub map: HashMap<GeoHashKey, Vec<UnixEpoch>>,
}

impl MappedQueryBuffer {
    pub fn new() -> Self {
        MappedQueryBuffer::default()
    }

    // !!このメソッドでは全くerror処理していない
    pub fn mapping(&mut self, query_buffer: &QueryBuffer) {
        let mut map: HashMap<GeoHashKey, Vec<UnixEpoch>> = HashMap::new();
        for query_rep in query_buffer.queries.iter() {
            for (geohash, unixepoch_vec) in query_rep.parameters.iter() {
                match map.get(geohash) {
                    Some(sorted_list) => { map.insert(*geohash, _sorted_merge(&sorted_list, unixepoch_vec)); },
                    None => { map.insert(*geohash, unixepoch_vec.to_vec()); },
                };
            }
        }
        self.map = map.iter().map(|(g, u)| (g.clone(), u.clone())).collect();

        // for query_rep in query_buffer.queries.iter() {
        //     for (geohash, unixepoch_vec) in query_rep.parameters.iter() {
        //         match self.map.get(geohash) {
        //             Some(sorted_list) => { self.map.insert(*geohash, _sorted_merge(&sorted_list, unixepoch_vec)); },
        //             None => { self.map.insert(*geohash, unixepoch_vec.to_vec()); },
        //         };
        //     }
        // }

        println!("[SGX] Q size {}", self.map.len());
    }
}