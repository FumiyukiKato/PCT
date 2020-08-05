use std::vec::Vec;
use std::collections::HashSet;

use primitive::*;
use encoded_query_buffer::EncodedQueryBuffer;
use encode_finite_state_transducer::EncodedFiniteStateTransducer;
use encoded_dictionary_buffer::EncodedDictionaryBuffer;
use encoded_result_buffer::EncodedResultBuffer;
use encoded_hash_table::EncodedHashTable;

#[derive(Clone, Debug)]
pub struct MappedEncodedQueryBuffer {
    pub map: EncodedFiniteStateTransducer,
    // pub map: EncodedHashTable,
}

impl MappedEncodedQueryBuffer {
    pub fn new() -> Self {
        MappedEncodedQueryBuffer {
            map: EncodedFiniteStateTransducer::new()
            // map: EncodedHashTable::new()
        }
    }

    // !!このメソッドでは全くerror処理していない
    pub fn mapping(&mut self, query_buffer: &EncodedQueryBuffer) {
        self.map.mapping(query_buffer);
    }

    pub fn intersect(&self, dictionary_buffer: &EncodedDictionaryBuffer, result: &mut EncodedResultBuffer) {
        self.map.intersect(dictionary_buffer, result);
    }

    pub fn show_size(&self) {
        self.map.calc_memory();
    }
}
