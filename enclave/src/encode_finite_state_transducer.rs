use fst::{Set};
use std::vec::Vec;

use primitive::*;
use constant::*;
use mapped_encoded_query_buffer::MappedEncodedQueryBuffer;
use encoded_result_buffer::EncodedResultBuffer;


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

    pub fn intersect(&self, mapped_query_buffer: &MappedEncodedQueryBuffer, result: &mut EncodedResultBuffer) {
        for encoded_value_vec in mapped_query_buffer.map.iter() {
            if self.map.contains(encoded_value_vec) {
                result.data.push(*encoded_value_vec);
            }
        }
    }

    pub fn build_dictionary_buffer(
        &mut self,
        bytes: Vec<u8>,
    ) {
        self.map = Set::from_bytes(bytes);
    }

    pub fn calc_memory(&self) {
        println!("[FSA] r_i size = {} bytes", self.map.as_ref().size());
    }
}
