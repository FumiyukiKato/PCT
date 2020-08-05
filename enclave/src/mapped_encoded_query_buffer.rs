use std::vec::Vec;
use std::collections::HashSet;

use primitive::*;
use encoded_query_buffer::EncodedQueryBuffer;

#[derive(Clone, Default, Debug)]
pub struct MappedEncodedQueryBuffer {
    pub map: Vec<EncodedValue>,
}

impl MappedEncodedQueryBuffer {
    pub fn new() -> Self {
        MappedEncodedQueryBuffer::default()
    }

    // !!このメソッドでは全くerror処理していない
    pub fn mapping(&mut self, query_buffer: &EncodedQueryBuffer) {
        let mut set: HashSet<EncodedValue> = HashSet::new();
        for query_rep in query_buffer.queries.iter() {
            for encoded_value in query_rep.parameters.iter() {
                set.insert(*encoded_value);
            }
        }
        self.map = set.into_iter().collect();

        println!("[SGX] Q size {}", self.map.len());
    }
}