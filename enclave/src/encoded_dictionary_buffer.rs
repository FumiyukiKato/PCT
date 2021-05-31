use std::vec::Vec;

use encoded_query_buffer::EncodedQueryBuffer;
use encoded_result_buffer::EncodedResultBuffer;
#[cfg(feature = "hashtable")]
use encoded_hash_table::EncodedHashTable;
// #[cfg(feature = "fsa")]
use fast_succinct_trie::FST;

// #[cfg(feature = "fsa")]
pub struct EncodedDictionaryBuffer {
    pub data: FST,
}
// #[cfg(feature = "fsa")]
impl EncodedDictionaryBuffer {
    pub fn new() -> Self {
        EncodedDictionaryBuffer {
            data: FST::new()            
        }
    }

    pub fn intersect(&self, query_buffer: &EncodedQueryBuffer, result: &mut EncodedResultBuffer) {
        self.data.intersect(query_buffer, result);
    }

    pub fn build_dictionary_buffer(
        &mut self,
        encoded_value_vec: Vec<u8>,
    ) {
        self.data.build_dictionary_buffer(encoded_value_vec);
    }

    pub fn show_size(&self) {
        self.data.calc_memory();
    }
}

#[cfg(feature = "hashtable")]
#[derive(Clone, Debug)]
pub struct EncodedDictionaryBuffer {
    pub data: EncodedHashTable,
}
#[cfg(feature = "hashtable")]
impl EncodedDictionaryBuffer {
    pub fn new() -> Self {
        EncodedDictionaryBuffer {
            data: EncodedHashTable::new()
        }
    }

    pub fn intersect(&self, mapped_query_buffer: &MappedEncodedQueryBuffer, result: &mut EncodedResultBuffer) {
        self.data.intersect(mapped_query_buffer, result);
    }

    pub fn build_dictionary_buffer(
        &mut self,
        encoded_value_vec: Vec<u8>,
    ) {
        self.data.build_dictionary_buffer(encoded_value_vec);
    }

    pub fn show_size(&self) {
        self.data.calc_memory();
    }
}