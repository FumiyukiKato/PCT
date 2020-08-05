use std::vec::Vec;

use primitive::*;
use constant::*;
use encoded_query_buffer::EncodedQueryBuffer;
use encoded_result_buffer::EncodedResultBuffer;
use encoded_hash_table::EncodedHashTable;
use encode_finite_state_transducer::EncodedFiniteStateTransducer;

#[derive(Clone, Debug)]
pub struct EncodedDictionaryBuffer {
    // pub data: EncodedFiniteStateTransducer,
    pub data: EncodedHashTable,
}

impl EncodedDictionaryBuffer {
    pub fn new() -> Self {
        EncodedDictionaryBuffer {
            // data: EncodedFiniteStateTransducer::new()            
            data: EncodedHashTable::new()
        }
    }

    pub fn intersect(&self, query_buffer: &EncodedQueryBuffer, result: &mut EncodedResultBuffer) {
        self.data.intersect(query_buffer, result);
    }

    pub fn build_dictionary_buffer(
        &mut self,
        encoded_value_vec: &Vec<u8>,
        size: usize,
    ) {
        self.data.build_dictionary_buffer(encoded_value_vec, size);
    }

    pub fn show_size(&self) {
        self.data.calc_memory();
    }
}