use std::vec::Vec;
use primitive::{ QueryId, EncodedValue };
use constant::*;

/* Type EncodedQueryRep */
#[derive(Clone, Default, Debug)]
pub struct EncodedQueryRep {
    pub id: QueryId,
    pub parameters: Vec<EncodedValue>,
}

impl EncodedQueryRep {
    pub fn new() -> Self {
        EncodedQueryRep {
            id: 0,
            parameters: Vec::with_capacity(QUERY_SIZE),
        }
    }
}