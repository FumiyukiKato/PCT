use succinct_trie::trie::{Trie, TrajectoryHash};
use std::vec::Vec;

use encoded_result_buffer::EncodedResultBuffer;
use encoded_query_buffer::EncodedQueryBuffer;

// queryデータの方に使います．
pub struct FST {
    pub map: Trie,
    #[cfg(feature = "nfp")]
    pub th: TrajectoryHash,
}

impl FST {
    pub fn intersect(&self, query_buffer: &EncodedQueryBuffer, result: &mut EncodedResultBuffer) {
        for encoded_value_vec in query_buffer.queries.iter() {
            if result.data.contains(&encoded_value_vec.id) {
                continue; 
            }
            for key in encoded_value_vec.parameters.iter() {
                #[cfg(feature = "st")]
                if self.map.contains(key) {
                    result.data.insert(encoded_value_vec.id);
                    continue;
                }
                #[cfg(feature = "nfp")]
                if self.map.accurate_search(key, &self.th) {
                    result.data.insert(encoded_value_vec.id);
                    continue;
                }
            }
        }
    }

    pub fn build_dictionary_buffer(
        bytes: Vec<u8>,
    ) -> Self {
        #[cfg(feature = "nfp")]
        let th = TrajectoryHash::new(7, 24, 7);
        // let th = TrajectoryHash::new(7, 21, 10);
        // let th = TrajectoryHash::new(8, 25, 14);
        // let th = TrajectoryHash::new(8, 24, 11);
        #[cfg(feature = "nfp")]
        return Self { map: Trie::deserialize(&bytes), th };
        #[cfg(feature = "st")]
        return Self { map: Trie::deserialize(&bytes) };
    }

    pub fn calc_memory(&self) {
        println!("[FSA] r_i size = {} bytes", self.map.byte_size());
    }
}
