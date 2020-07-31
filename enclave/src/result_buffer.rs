use std::vec::Vec;

use constant::*;
use primitive::*;
use query_result::QueryResult;
use query_buffer::QueryBuffer;

/* Type ResultBuffer */
#[derive(Clone, Default, Debug)]
pub struct ResultBuffer {
    pub data: Vec<(GeoHashKey, UnixEpoch)>
}

impl ResultBuffer {
    pub fn new() -> Self {
        ResultBuffer::default()
    }

    // matchがネストして読みにくくなってしまっている
    // メソッドチェーンでもっと関数型っぽく書けば読みやすくなりそうではある
    pub fn build_query_response(
        &self,
        query_buffer: &QueryBuffer,
        response_vec: &mut Vec<u8>,
    ) {
        for query in query_buffer.queries.iter() {
            let mut result = QueryResult::new();
            result.query_id = query.id;
            for (geohash, unixepoch) in self.data.iter() {
                let is_exist = match query.parameters.get(geohash) {
                    Some(sorted_list) => { 
                        match sorted_list.binary_search(unixepoch) {
                            Ok(_) => true,
                            Err(_) => false,
                        }
                    },
                    None => { false },
                };
                if is_exist {
                    result.risk_level = 1;
                    break;
                }
            }
            response_vec.extend_from_slice(&result.to_be_bytes());
        }
    }
}