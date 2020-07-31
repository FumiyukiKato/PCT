use std::vec::Vec;
use std::collections::HashMap;

use primitive::{ QueryId, GeoHashKey, UnixEpoch };

/* Type QueryRep */
#[derive(Clone, Default, Debug)]
pub struct QueryRep {
    pub id: QueryId,
    pub parameters: HashMap<GeoHashKey, Vec<UnixEpoch>>
}

// idの正しさは呼び出し側が責任を持つ
impl QueryRep {
    pub fn new() -> Self {
        QueryRep::default()
    }
}