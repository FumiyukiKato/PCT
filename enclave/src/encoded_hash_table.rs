use std::vec::Vec;
use std::collections::HashSet;
use std::mem;
use primitive::*;
use constant::*;
use encoded_result_buffer::EncodedResultBuffer;
use encoded_dictionary_buffer::EncodedDictionaryBuffer;
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

    pub fn intersect(&self, dictionary_buffer: &EncodedDictionaryBuffer, result: &mut EncodedResultBuffer) {
        for encoded_value_vec in dictionary_buffer.data.iter() {
            if self.map.contains(encoded_value_vec) {
                result.data.push(*encoded_value_vec);
            }
        }

        // for (dict_geohash, dict_unixepoch_vec) in self.map.iter() {
        //     match mapped_query_buffer.map.get(dict_geohash) {
        //         Some(query_unixepoch_vec) => {
        //             Self::judge_contact(query_unixepoch_vec, dict_unixepoch_vec, dict_geohash, result);
        //         },
        //         None => {}
        //     }
        // }
    }

    pub fn mapping(&mut self, query_buffer: &EncodedQueryBuffer) {
        for query_rep in query_buffer.queries.iter() {
            for encoded_value in query_rep.parameters.iter() {
                self.map.insert(*encoded_value);
            }
        }
        self.map.shrink_to(self.map.len());
        println!("[SGX] unique query size {}", self.map.len());
    }

    pub fn calc_memory(&self) -> usize {
        return (self.map.capacity() * 11 / 10) * (mem::size_of::<EncodedValue>() + mem::size_of::<()>() + mem::size_of::<u64>());
    }
}