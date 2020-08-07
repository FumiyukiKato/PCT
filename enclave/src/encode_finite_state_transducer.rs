use fst::{Set};
use std::vec::Vec;
use std::collections::HashSet;
use std::collections::BTreeSet;

use primitive::*;
use constant::*;
use encoded_dictionary_buffer::EncodedDictionaryBuffer;
use encoded_result_buffer::EncodedResultBuffer;
use encoded_query_buffer::EncodedQueryBuffer;


#[derive(Clone, Debug, Default)]
pub struct FstValue { pub value: EncodedValue }

impl FstValue {
    pub fn new(vec: EncodedValue) -> Self {
        FstValue { value: vec }
    }
}

impl AsRef<[u8]> for FstValue {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.value
    }
}

// queryデータの方に使います．
#[derive(Clone, Debug)]
pub struct EncodedFiniteStateTransducer {
    pub map: Set<Vec<u8>>,
}

impl EncodedFiniteStateTransducer {
    pub fn new() -> Self {
        EncodedFiniteStateTransducer {
            map: Set::from_iter(Vec::<FstValue>::new()).unwrap()
        }
    }

    pub fn from_vec(encoded_value_vec: &Vec<EncodedValue>) -> Self {
        EncodedFiniteStateTransducer {
            map: Set::from_iter(encoded_value_vec).unwrap()
        }
    }

    pub fn mapping(&mut self, query_buffer: &EncodedQueryBuffer) {
        let mut set: BTreeSet<EncodedValue> = BTreeSet::new();
        for query_rep in query_buffer.queries.iter() {
            for encoded_value in query_rep.parameters.iter() {
                set.insert(*encoded_value);
            }
        }
        println!("[SGX] unique query size {}", set.len());
        self.map = Set::from_iter(set.into_iter().collect::<Vec<EncodedValue>>()).unwrap();
    }

    pub fn intersect(&self, dictionary_buffer: &EncodedDictionaryBuffer, result: &mut EncodedResultBuffer) {
        for encoded_value_vec in dictionary_buffer.data.iter() {
            if self.map.contains(encoded_value_vec) {
                result.data.insert(*encoded_value_vec);
            }
        }
    }

    pub fn calc_memory(&self) -> usize {
        return self.map.as_ref().size()
    }
}
