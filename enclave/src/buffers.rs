use std::string::String;
use std::vec::Vec;
use std::cell::RefCell;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::collections::HashMap;

pub const UNIXEPOCH_U8_SIZE: usize = 10;
pub const GEOHASH_U8_SIZE: usize = 10;
pub const QUERY_U8_SIZE: usize = UNIXEPOCH_U8_SIZE + GEOHASH_U8_SIZE;
// risk_level 1バイト + qeuryId
pub const QUERY_ID_SIZE_U8: usize = 8;
pub const QUERY_RESULT_U8: usize = 1;
pub const RESPONSE_DATA_SIZE_U8: usize = QUERY_ID_SIZE_U8 + QUERY_RESULT_U8;

pub const THREASHOLD: usize = 100000;

pub const CONTACT_TIME_THREASHOLD: u64 = 600;

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
        DictionaryBuffer {
            data: HashMap::with_capacity(THREASHOLD)
        }
    }

    // dictinary側の方がサイズが大きい場合と小さい場合がある
    // 一般的なケースだとdictinary側の方が大きい？
    // 計算量はMかNのどっちかになるのでどちらも実装しておく
    /* dictinaryの合計サイズの方が大きい場合はこれを採用した方が早いけど逆なら改善可能 */
    pub fn intersect(&self, mapped_query_buffer: &MappedQueryBuffer, result: &mut ResultBuffer) {
        for (dict_geohash, dict_unixepoch_vec) in self.data.iter() {
            match mapped_query_buffer.map.get(dict_geohash) {
                Some(query_unixepoch_vec) => {
                    self.judge_contact(query_unixepoch_vec, dict_unixepoch_vec, dict_geohash, result);
                },
                None => {}
            }
        }
    }

    /* CONTACT_TIME_THREASHOLDの幅で接触を判定して結果をResultBufferに返す */
    /* resultbufferにいれるところまでやるのは分かりにくいけど，パフォーマンス的にそうする */
    fn judge_contact(&self, query_unixepoch_vec: &Vec<UnixEpoch>, dict_unixepoch_vec: &Vec<UnixEpoch>, geohash: &GeoHashKey, result: &mut ResultBuffer) {
        let mut finish: bool = false;
        for query_unixepoch in query_unixepoch_vec.iter() {
            let last = dict_unixepoch_vec.len() - 1;
            for (i, dict_unixepoch) in dict_unixepoch_vec.iter().enumerate() {
                if *dict_unixepoch > CONTACT_TIME_THREASHOLD + *query_unixepoch {
                    break;
                } else if (*dict_unixepoch < CONTACT_TIME_THREASHOLD + *query_unixepoch) && (*query_unixepoch < CONTACT_TIME_THREASHOLD + *dict_unixepoch) {
                    result.data.push((*geohash ,*query_unixepoch));
                } else {
                    if i == last {
                        finish = true;
                    }
                    continue;
                }
            }
            if finish { break; }
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

// #[derive(Clone, Default, Debug)]
// pub struct Period {
//     pub array: Vec<(UnixEpoch, UnixEpoch)>
// }

// impl Period {
//     pub fn new() -> Self {
//         Period::default()
//     }
// }

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

/* 
Type QueryResult 
    バイトへのシリアライズを担当するよ
*/
#[derive(Clone, Default, Debug)]
pub struct QueryResult {
    pub query_id: QueryId,
    pub risk_level: u8,
    pub result_vec: Vec<(GeoHashKey, UnixEpoch)>,
}

impl QueryResult {
    pub fn new() -> Self {
        return QueryResult {
            query_id: 1,
            risk_level: 0,
            result_vec: vec![],
        }
    }

    pub fn to_be_bytes(&self) -> [u8; RESPONSE_DATA_SIZE_U8] {
        let mut res = [0; RESPONSE_DATA_SIZE_U8];
        res[..QUERY_ID_SIZE_U8].clone_from_slice(&self.query_id.to_be_bytes());
        res[RESPONSE_DATA_SIZE_U8-QUERY_RESULT_U8] = self.risk_level;
        res
    }
}

/* 
SGXのステート
    ステートは全部グローバル変数に持ってヒープにメモリを確保する
*/
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
