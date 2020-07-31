use std::vec::Vec;

use constant::*;
use query_rep::QueryRep;
use utils::*;
use primitive::*;

/* Type QueryBuffer [q_1,...q_{N_c}]
    このデータ構造は基本的に固定して良いはず
*/
#[derive(Clone, Default, Debug)]
pub struct QueryBuffer {
    pub queries: Vec<QueryRep>,
}

impl QueryBuffer {
    pub fn new() -> Self {
        QueryBuffer::default()
    }

    // !!このメソッドでは全くerror処理していない
    // queryを個々に組み立ててbufferに保持する
    pub fn build_query_buffer(
        &mut self,
        total_query_data_vec: &Vec<u8>,
        size_list_vec       : &Vec<usize>,
        query_id_list_vec   : &Vec<u64>,
    ) -> i8 {
        let mut cursor = 0;
        for i in 0_usize..(size_list_vec.len()) {
            let size: usize = size_list_vec[i]*QUERY_U8_SIZE;
            let this_query = &total_query_data_vec[cursor..cursor+size];
            cursor = cursor+size; // 忘れないようにここで更新
            
            let mut query = QueryRep::new();
            query.id = query_id_list_vec[i];
            for i in 0_usize..(size/QUERY_U8_SIZE) {
                let mut timestamp = [0_u8; UNIXEPOCH_U8_SIZE];
                let mut geo_hash = [0_u8; GEOHASH_U8_SIZE];
                timestamp.copy_from_slice(&this_query[i*QUERY_U8_SIZE..i*QUERY_U8_SIZE+UNIXEPOCH_U8_SIZE]);
                geo_hash.copy_from_slice(&this_query[i*QUERY_U8_SIZE+UNIXEPOCH_U8_SIZE..(i + 1)*QUERY_U8_SIZE]);
                match query.parameters.get_mut(&geo_hash) {
                    Some(sorted_list) => { _sorted_push(sorted_list, unixepoch_from_u8(timestamp)) },
                    None => { query.parameters.insert(geo_hash as GeoHashKey, vec![unixepoch_from_u8(timestamp)]); },
                };
            }
            self.queries.push(query);
        }
        return 0;
    }
}