use std::vec::Vec;

use primitive::*;
use constant::*;

#[derive(Clone, Debug)]
pub struct EncodedDictionaryBuffer {
    pub data: Vec<EncodedValue>,
}

impl EncodedDictionaryBuffer {
    pub fn new() -> Self {
        EncodedDictionaryBuffer {
            data: Vec::<EncodedValue>::new()
        }
    }

    pub fn build_dictionary_buffer(
        &mut self,
        encoded_value_vec: &Vec<u8>,
        size: usize,
    ) {
        for i in 0usize..(size) {
            let mut encoded_value: EncodedValue = [0_u8; ENCODEDVALUE_SIZE];
            encoded_value.copy_from_slice(&encoded_value_vec[ENCODEDVALUE_SIZE*i..ENCODEDVALUE_SIZE*(i+1)]);
            self.data.push(encoded_value);
        }
    }
}