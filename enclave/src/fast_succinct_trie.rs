use succinct_trie::trie::Trie;
use std::vec::Vec;

use encoded_result_buffer::EncodedResultBuffer;
use encoded_query_buffer::EncodedQueryBuffer;

// queryデータの方に使います．
pub struct FST {
    pub map: Trie,
}

impl FST {
    pub fn intersect(&self, query_buffer: &EncodedQueryBuffer, result: &mut EncodedResultBuffer) {
        for encoded_value_vec in query_buffer.queries.iter() {
            if result.data.contains(&encoded_value_vec.id) {
                continue; 
            }
            for key in encoded_value_vec.parameters.iter() {
                if self.map.contains(key) {
                    result.data.insert(encoded_value_vec.id);
                    continue;
                }
            }
        }
    }

    pub fn build_dictionary_buffer(
        bytes: Vec<u8>,
    ) -> Self {
        Self { map: Trie::deserialize(&bytes) }
    }

    pub fn calc_memory(&self) {
        println!("[FSA] r_i size = {} bytes", self.map.byte_size());
    }
}
