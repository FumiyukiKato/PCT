use std::string::String;
use std::vec::Vec;
use std::cell::RefCell;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::collections::HashMap;

pub const UNIXEPOCH_U8_SIZE: usize = 10;
pub const GEOHASH_U8_SIZE: usize = 10;
pub const QUERY_U8_SIZE: usize = UNIXEPOCH_U8_SIZE + GEOHASH_U8_SIZE;
pub const RESPONSE_DATA_SIZE_U8: usize = 1;

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
    data.vec<Unixepoch>がソート済みでユニークセットになっていることは呼び出し側が保証している
*/
#[derive(Clone, Default, Debug)]
pub struct DictionaryBuffer {
    pub data: PCTDataStructure
}

impl DictionaryBuffer {
    pub fn new() -> Self {
        DictionaryBuffer::default()
    }

    pub fn intersect(&self, mapped_query_buffer: &MappedQueryBuffer, result: &mut ResultBuffer) {
        for (query_geohash, query_unixepoch_vec) in mapped_query_buffer.map.iter() {
            match self.data.get(query_geohash) {
                Some(dic_unixepoch_list) => {
                    for query_unixepoch in query_unixepoch_vec.iter() {
                        if dic_unixepoch_list.contains(query_unixepoch) {
                            result.data.push((*query_geohash, *query_unixepoch));
                        }
                    }
                },
                None => {}
            }
        }
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
    pub parameters: HashMap<GeoHashKey, Vec<UnixEpoch>>
}

// idの正しさは呼び出し側が責任を持つ
impl QueryRep {
    pub fn new() -> Self {
        QueryRep::default()
    }
}

/* 
Type MappedQueryBuffer 
    こっちのクエリ側のデータ構造も変わる可能性がある
    いい感じに抽象化するのがめんどくさいのでこのデータ構造自体を変える
    map.vec<Unixepoch>がソート済みでユニークセットになっていることは呼び出し側が保証している
*/
#[derive(Clone, Default, Debug)]
pub struct MappedQueryBuffer {
    pub map: HashMap<GeoHashKey, Vec<UnixEpoch>>,
}

impl MappedQueryBuffer {
    pub fn new() -> Self {
        MappedQueryBuffer::default()
    }
}

// バファリングするクエリはせいぜい10000なので64bitで余裕
pub type QueryId = u64;

/* Type QueryBuffer */
#[derive(Clone, Default, Debug)]
pub struct QueryBuffer {
    pub queries: Vec<QueryRep>,
}

impl QueryBuffer {
    pub fn new() -> Self {
        QueryBuffer::default()
    }
}

/* Type QueryResult */
#[derive(Clone, Default, Debug)]
pub struct QueryResult {
    pub risk_level: i8,
    pub result_vec: Vec<(GeoHashKey, UnixEpoch)>,
}

impl QueryResult {
    pub fn new() -> Self {
        QueryResult::default()
    }
}


/* 
SGXのステート
    ステートは全部グローバル変数に持ってヒープにメモリを確保する
*/
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
pub fn get_ref_mapped_query_buffer() -> Option<&'static RefCell<MappedQueryBuffer>> {
    let ptr = MAPPED_QUERY_BUFFER.load(Ordering::SeqCst) as * mut RefCell<MappedQueryBuffer>;
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
