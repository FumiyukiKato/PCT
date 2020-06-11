use std::string::String;
use std::vec::Vec;
use std::cell::RefCell;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::collections::HashMap;

pub const UNIXEPOCH_U8_SIZE: usize = 10;
pub const GEOHASH_U8_SIZE: usize = 10;
pub const QUERY_U8_SIZE: usize = UNIXEPOCH_U8_SIZE + GEOHASH_U8_SIZE;

/* ################################ */

/* 
PCTに最適化したデータ構造 
ジェネリクスじゃなくてここで直接データ構造を変える 
*/
type PCTDataStructure = HashMap<GeoHashKey, Vec<UnixEpoch>>;

// type PCTDataStructure = BloomFilter<GeoHashKey, Period>;みたいな






/* ################################ */


/* 
Type key for data structures (= GeoHash) 
とりあえず無難にStringにしておく，あとで普通に[u8; 8]とかに変える[u64;2]とかでも良さそう？
*/
pub type GeoHashKey = [u8; GEOHASH_U8_SIZE];

/* 
Type DictionaryBuffer 
シーケンシャルな読み込みのためのバッファ，サイズを固定しても良い 
*/
#[derive(Clone, Default, Debug)]
pub struct DictionaryBuffer {
    pub data: PCTDataStructure
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
pub fn unixepoch_from_u8(u_timestamp: [u8; UNIXEPOCH_U8_SIZE]) -> UnixEpoch {
    let s_timestamp = String::from_utf8(u_timestamp.to_vec()).unwrap();
    let num: UnixEpoch = (&s_timestamp).parse().unwrap();
    num
}

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

/* 
Type MappedQuery 
こっちのクエリ側のデータ構造も変わる可能性がある
いい感じに抽象化するのがめんどくさいのでこのデータ構造自体を変える
*/
#[derive(Clone, Default, Debug)]
pub struct MappedQuery {
    pub map: HashMap<GeoHashKey, Vec<UnixEpoch>>,
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
