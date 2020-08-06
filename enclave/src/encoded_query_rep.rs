use std::collections::HashSet;
use primitive::{ QueryId, EncodedValue };
use constant::*;

/* Type EncodedQueryRep */
#[derive(Clone, Default, Debug)]
pub struct EncodedQueryRep {
    pub id: QueryId,
    pub parameters: HashSet<EncodedValue>,
}

impl EncodedQueryRep {
    pub fn new() -> Self {
        EncodedQueryRep {
            id: 0,
            parameters: HashSet::with_capacity(QUERY_SIZE),
        }
    }
}