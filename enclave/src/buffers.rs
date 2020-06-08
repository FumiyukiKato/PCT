use std::string::String;
use std::vec::Vec;
use std::cell::RefCell;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::collections::HashMap;

pub const UNIXEPOCH_U8_SIZE: usize = 10;
pub const GEOHASH_U8_SIZE: usize = 10;
pub const QUERY_U8_SIZE: usize = UNIXEPOCH_U8_SIZE + GEOHASH_U8_SIZE;

/* Type key for data structures (= GeoHash) */
// とりあえず無難にStringにしておく，あとで普通に[u8; 8]とかに変える[u64;2]とかでも良さそう？
pub type GeoHashKey = String;

/* Type DictionaryBuffer */
#[derive(Clone, Default, Debug)]
pub struct DictionaryBuffer {
    pub data: HashMap<GeoHashKey, Period>
}

impl DictionaryBuffer {
    pub fn new() -> Self {
        DictionaryBuffer::default()
    }
}

/* Type ResultBuffer */
#[derive(Clone, Default, Debug)]
pub struct ResultBuffer {
    pub data: Vec<(GeoHashKey, UnixEpoch)>
}

impl ResultBuffer {
    pub fn new() -> Self {
        ResultBuffer::default()
    }
}

/* Type Period */
pub type UnixEpoch = u64;
#[derive(Clone, Default, Debug)]
pub struct Period {
    pub array: Vec<(UnixEpoch, UnixEpoch)>
}

impl Period {
    pub fn new() -> Self {
        Period::default()
    }
}

/* Type QueryRep */
#[derive(Clone, Default, Debug)]
pub struct QueryRep {
    pub id: QueryId,
    pub parameters: Vec<([u8; GEOHASH_U8_SIZE], [u8; UNIXEPOCH_U8_SIZE])>,
}

// idの正しさは呼び出し側が責任を持つ
impl QueryRep {
    pub fn new() -> Self {
        QueryRep::default()
    }
}

/* Type MappedQuery */
#[derive(Clone, Default, Debug)]
pub struct MappedQuery {
    pub array: Vec<(GeoHashKey, Period)>,
}

impl MappedQuery {
    pub fn new() -> Self {
        MappedQuery::default()
    }
}

// バファリングするクエリはせいぜい10000なので64bitで余裕
pub type QueryId = u64;

/* Type QueryBuffer */
#[derive(Clone, Default, Debug)]
pub struct QueryBuffer {
    pub queries: HashMap<QueryId, QueryRep>,
}

impl QueryBuffer {
    pub fn new() -> Self {
        QueryBuffer::default()
    }
}

pub static DICTIONARY_BUFFER: AtomicPtr<()> = AtomicPtr::new(0 as * mut ());
pub fn get_ref_dictionary_buffer() -> Option<&'static RefCell<DictionaryBuffer>> {
    let ptr = DICTIONARY_BUFFER.load(Ordering::SeqCst) as * mut RefCell<DictionaryBuffer>;
    if ptr.is_null() {
        None
    } else {
        Some(unsafe { &* ptr })
    }
}

pub static QUERY_BUFFER: AtomicPtr<()> = AtomicPtr::new(0 as * mut ());
pub fn get_ref_query_buffer() -> Option<&'static RefCell<QueryBuffer>> {
    let ptr = QUERY_BUFFER.load(Ordering::SeqCst) as * mut RefCell<QueryBuffer>;
    if ptr.is_null() {
        None
    } else {
        Some(unsafe { &* ptr })
    }
}

pub static MAPPED_QUERY_BUFFER: AtomicPtr<()> = AtomicPtr::new(0 as * mut ());
pub fn get_ref_mapped_query_buffer() -> Option<&'static RefCell<MappedQuery>> {
    let ptr = MAPPED_QUERY_BUFFER.load(Ordering::SeqCst) as * mut RefCell<MappedQuery>;
    if ptr.is_null() {
        None
    } else {
        Some(unsafe { &* ptr })
    }
}

pub static RESULT_BUFFER: AtomicPtr<()> = AtomicPtr::new(0 as * mut ());
pub fn get_ref_result_buffer() -> Option<&'static RefCell<ResultBuffer>> {
    let ptr = RESULT_BUFFER.load(Ordering::SeqCst) as * mut RefCell<ResultBuffer>;
    if ptr.is_null() {
        None
    } else {
        Some(unsafe { &* ptr })
    }
}
