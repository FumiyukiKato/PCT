use encoded_query_rep::EncodedQueryRep;
use constant::*;
use std::vec::Vec;

#[derive(Clone, Default, Debug)]
pub struct EncodedQueryBuffer {
    pub queries: Vec<EncodedQueryRep>,
}

impl EncodedQueryBuffer {
    pub fn new() -> Self {
        EncodedQueryBuffer::default()
    }

    // !!このメソッドでは全くerror処理していない
    // queryを個々に組み立ててbufferに保持する
    pub fn build_query_buffer(
        &mut self,
        total_query_data_vec: Vec<u8>,
        query_id_list_vec   : Vec<u64>,
    ) -> i8 {
        for i in 0_usize..(query_id_list_vec.len()) {
            let mut query = EncodedQueryRep::new();
            query.id = query_id_list_vec[i];
            for j in 0_usize..QUERY_SIZE {
                let mut encoded_value = [0_u8; ENCODEDVALUE_SIZE];
                encoded_value.copy_from_slice(&total_query_data_vec[(i*QUERY_SIZE+j)*ENCODEDVALUE_SIZE..(i*QUERY_SIZE+j+1)*ENCODEDVALUE_SIZE]);
                query.parameters.push(encoded_value);
            }
            self.queries.push(query);
        }
        return 0;
    }
}