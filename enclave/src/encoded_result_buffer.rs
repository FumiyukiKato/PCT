use std::vec::Vec;

use primitive::*;
use encoded_query_buffer::EncodedQueryBuffer;
use query_result::QueryResult;

#[derive(Clone, Default, Debug)]
pub struct EncodedResultBuffer {
    pub data: Vec<QueryId>,
}

impl EncodedResultBuffer {
    pub fn new() -> Self {
        EncodedResultBuffer::default()
    }

    // matchがネストして読みにくくなってしまっている
    // メソッドチェーンでもっと関数型っぽく書けば読みやすくなりそうではある
    pub fn build_query_response(
        &self,
        query_buffer: &EncodedQueryBuffer,
        response_vec: &mut Vec<u8>,
    ) {
        for query in query_buffer.queries.iter() {
            let mut result = QueryResult::new();
            result.query_id = query.id;
            if self.data.contains(&query.id) {
                result.risk_level = 1;
            }
            response_vec.extend_from_slice(&result.to_be_bytes());
        }
    }
}