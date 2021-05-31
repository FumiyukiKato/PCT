use std::vec::Vec;
use std::collections::HashSet;
use std::mem;
use bincode;
use primitive::*;
use constant::*;
use encoded_result_buffer::EncodedResultBuffer;
use encoded_query_buffer::EncodedQueryBuffer;

#[derive(Clone, Default, Debug)]
pub struct EncodedHashTable {
    pub map: HashSet<EncodedValue>,
}
impl EncodedHashTable {
    pub fn new() -> Self {
        EncodedHashTable {
            map: HashSet::with_capacity(THREASHOLD)
        }
    }

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
        &mut self,
        bytes: Vec<u8>,
    ) {
        self.map = bincode::deserialize(&bytes[..]).unwrap();
    }

    pub fn calc_memory(&self) {
        println!("[HashTable] r_i size = {} bytes", (self.map.capacity() * 11 / 10) * (mem::size_of::<EncodedValue>() + mem::size_of::<()>() + mem::size_of::<u64>()));
    }
}