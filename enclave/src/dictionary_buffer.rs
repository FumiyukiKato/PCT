use std::vec::Vec;

use geohash_table::GeohashTable;
use mapped_query_buffer::MappedQueryBuffer;
use result_buffer::ResultBuffer;

/* 
Type DictionaryBuffer "R"
    シーケンシャルな読み込みのためのバッファ，サイズを固定しても良い
    data.vec<Unixepoch>がソート済みでユニークセットになっていることは呼び出し側が保証している
*/
#[derive(Clone, Default, Debug)]
pub struct DictionaryBuffer {
    pub data: GeohashTable
}

impl DictionaryBuffer {
    pub fn new() -> Self {
        DictionaryBuffer::default()
    }

    pub fn intersect(&self, mapped_query_buffer: &MappedQueryBuffer, result: &mut ResultBuffer) {
        self.data.intersect(mapped_query_buffer, result);
    }

    pub fn build_dictionary_buffer(
        &mut self,
        geohash_data_vec: &Vec<u8>,
        unixepoch_data_vec: &Vec<u64>,
        size_list_vec: &Vec<usize>,
    ) {
        self.data.build_dictionary_buffer(geohash_data_vec, unixepoch_data_vec, size_list_vec);
    }

    // pub fn build_fst_dictionary_buffer(
    //     &mut self,
    //     encoded_value_vec: &Vec<u8>,
    //     size: usize,
    // ) {
    //     self.data.build_dictionary_buffer(encoded_value_vec, size);
    // }
}