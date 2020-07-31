use std::collections::HashSet;

use primitive::{ QueryId, EncodedValue };

/* Type EncodedQueryRep */
#[derive(Clone, Default, Debug)]
pub struct EncodedQueryRep {
    pub id: QueryId,
    pub parameters: HashSet<EncodedValue>,
}

impl EncodedQueryRep {
    pub fn new() -> Self {
        EncodedQueryRep::default()
    }
}