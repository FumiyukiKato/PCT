use crate::config::*;

pub struct BitvectorSuffix {}


impl BitvectorSuffix {
    pub fn construct_suffix(suffix_type: SuffixType) -> word_t {
        return match suffix_type {
            _ => 0
        }
    }

    pub fn new<'a>() -> BitvectorSuffix {
        BitvectorSuffix {}
    }
}