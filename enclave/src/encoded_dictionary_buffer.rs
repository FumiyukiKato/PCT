use std::vec::Vec;

use mapped_encoded_query_buffer::MappedEncodedQueryBuffer;
use encoded_result_buffer::EncodedResultBuffer;
use encoded_hash_table::EncodedHashTable;
use encode_finite_state_transducer::EncodedFiniteStateTransducer;

#[derive(Clone, Debug)]
pub struct EncodedDictionaryBuffer {
    // pub data: EncodedHashTable
    pub data: EncodedFiniteStateTransducer
}

impl EncodedDictionaryBuffer {
    pub fn new() -> Self {
        EncodedDictionaryBuffer {
            // data: EncodedHashTable::new()
            data: EncodedFiniteStateTransducer::new()
        }
    }

    pub fn intersect(&self, mapped_query_buffer: &MappedEncodedQueryBuffer, result: &mut EncodedResultBuffer) {
        self.data.intersect(mapped_query_buffer, result);
    }

    pub fn build_dictionary_buffer(
        &mut self,
        encoded_value_vec: &Vec<u8>,
        size: usize,
    ) {
        self.data.build_dictionary_buffer(encoded_value_vec, size);
    }
}