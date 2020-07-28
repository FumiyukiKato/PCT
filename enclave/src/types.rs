use std::string::String;
use std::vec::Vec;
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

// UNIX EPOCH INTERVAL OF THE GPS DATA
pub const TIME_INTERVAL: u64 = 600;

/* 
Type key for data structures (= GeoHash) 
    とりあえず無難にStringにしておく，あとで普通に[u8; 8]とかに変える[u64;2]とかでも良さそう？
*/
pub type GeoHashKey = [u8; GEOHASH_U8_SIZE];

pub type UnixEpoch = u64;
pub fn unixepoch_from_u8(u_timestamp: [u8; UNIXEPOCH_U8_SIZE]) -> UnixEpoch {
    let s_timestamp = String::from_utf8(u_timestamp.to_vec()).unwrap();
    let num: UnixEpoch = (&s_timestamp).parse().unwrap();
    num
}

// バファリングするクエリはせいぜい10000なので64bitで余裕
pub type QueryId = u64;

/* 
Type DictionaryBuffer "R"
    シーケンシャルな読み込みのためのバッファ，サイズを固定しても良い
    data.vec<Unixepoch>がソート済みでユニークセットになっていることは呼び出し側が保証している
*/
#[derive(Clone, Default, Debug)]
pub struct DictionaryBuffer {
    pub data: GeohashTable
}

impl DictionaryBuffer {
    pub fn new() -> Self {
        DictionaryBuffer::default()
    }

    pub fn intersect(&self, mapped_query_buffer: &MappedQueryBuffer, result: &mut ResultBuffer) {
        self.data.intersect(mapped_query_buffer, result);
    }

    pub fn build_dictionary_buffer(
        &mut self,
        geohash_data_vec: &Vec<u8>,
        unixepoch_data_vec: &Vec<u64>,
        size_list_vec: &Vec<usize>,
    ) {
        self.data.build_dictionary_buffer(geohash_data_vec, unixepoch_data_vec, size_list_vec);
    }
}

/* 
Type MappedQueryBuffer "Q"
    こっちのクエリ側のデータ構造も変わる可能性がある
    いい感じに抽象化するのがめんどくさいのでこのデータ構造自体を変える
    map.vec<Unixepoch>がソート済みでユニークセットになっていることは呼び出し側が保証している
*/
#[derive(Clone, Default, Debug)]
pub struct MappedQueryBuffer {
    pub map: Vec<(GeoHashKey, Vec<UnixEpoch>)>,
    // pub map: HashMap<GeoHashKey, Vec<UnixEpoch>>,
}

impl MappedQueryBuffer {
    pub fn new() -> Self {
        MappedQueryBuffer::default()
    }

    // !!このメソッドでは全くerror処理していない
    pub fn from_query_buffer(&mut self, query_buffer: &QueryBuffer) {
        let mut map: HashMap<GeoHashKey, Vec<UnixEpoch>> = HashMap::new();
        for query_rep in query_buffer.queries.iter() {
            for (geohash, unixepoch_vec) in query_rep.parameters.iter() {
                match map.get(geohash) {
                    Some(sorted_list) => { map.insert(*geohash, _sorted_merge(&sorted_list, unixepoch_vec)); },
                    None => { map.insert(*geohash, unixepoch_vec.to_vec()); },
                };
            }
        }
        self.map = map.iter().map(|(g, u)| (g.clone(), u.clone())).collect();

        // for query_rep in query_buffer.queries.iter() {
        //     for (geohash, unixepoch_vec) in query_rep.parameters.iter() {
        //         match self.map.get(geohash) {
        //             Some(sorted_list) => { self.map.insert(*geohash, _sorted_merge(&sorted_list, unixepoch_vec)); },
        //             None => { self.map.insert(*geohash, unixepoch_vec.to_vec()); },
        //         };
        //     }
        // }

        println!("[SGX] Q size {}", self.map.len());
    }
}

/* 
    PCTに最適化したデータ構造 
    ジェネリクスじゃなくてここに直接書く
*/
#[derive(Clone, Default, Debug)]
pub struct GeohashTable {
    pub map: HashMap<GeoHashKey, Vec<UnixEpoch>>
}
impl GeohashTable {
    pub fn new() -> Self {
        GeohashTable {
            map: HashMap::with_capacity(THREASHOLD)
        }
    }

    // dictinary側の方がサイズが大きい場合と小さい場合がある
    // 一般的なケースだとdictinary側の方が大きい？
    // 計算量はMかNのどっちかになるのでどちらも実装しておく
    /* dictinaryの合計サイズの方が大きい場合はこれを採用した方が早いけど逆なら改善可能 */
    pub fn intersect(&self, mapped_query_buffer: &MappedQueryBuffer, result: &mut ResultBuffer) {
        for (query_geohash, query_unixepoch_vec) in mapped_query_buffer.map.iter() {
            match self.map.get(query_geohash) {
                Some(dict_unixepoch_vec) => {
                    Self::judge_contact(query_unixepoch_vec, dict_unixepoch_vec, query_geohash, result);
                },
                None => {}
            }
        }

        // for (dict_geohash, dict_unixepoch_vec) in self.map.iter() {
        //     match mapped_query_buffer.map.get(dict_geohash) {
        //         Some(query_unixepoch_vec) => {
        //             Self::judge_contact(query_unixepoch_vec, dict_unixepoch_vec, dict_geohash, result);
        //         },
        //         None => {}
        //     }
        // }
    }

    /* CONTACT_TIME_THREASHOLDの幅で接触を判定して結果をResultBufferに返す */
    /* resultbufferにいれるところまでやるのは分かりにくいけど，パフォーマンス的にそうする */
    fn judge_contact(query_unixepoch_vec: &Vec<UnixEpoch>, dict_unixepoch_vec: &Vec<UnixEpoch>, geohash: &GeoHashKey, result: &mut ResultBuffer) {
        let last_index_of_query_unixepoch_vec = query_unixepoch_vec.len() - 1;
        let last_index_of_dict_unixepoch_vec = dict_unixepoch_vec.len() - 1;
        let mut i = 0;
        let mut j = 0;

        while i <= last_index_of_query_unixepoch_vec && j <= last_index_of_dict_unixepoch_vec {
            if dict_unixepoch_vec[j] - CONTACT_TIME_THREASHOLD < query_unixepoch_vec[i] && query_unixepoch_vec[i] < dict_unixepoch_vec[j] + CONTACT_TIME_THREASHOLD {
                result.data.push((*geohash, query_unixepoch_vec[i]));
                i += 1;
            } else if query_unixepoch_vec[i] < dict_unixepoch_vec[j] - CONTACT_TIME_THREASHOLD {
                i += 1;
            } else {
                j += 1;
            }
        }
    }

    fn build_dictionary_buffer(
        &mut self,
        geohash_data_vec: &Vec<u8>,
        unixepoch_data_vec: &Vec<u64>,
        size_list_vec: &Vec<usize>,
    ) {
        let mut cursor: usize = 0;
        for i in 0usize..(size_list_vec.len()) {
            let mut geohash = GeoHashKey::default(); 
            geohash.copy_from_slice(&geohash_data_vec[GEOHASH_U8_SIZE*i..GEOHASH_U8_SIZE*(i+1)]);
            // centralデータはすでにsorted unique listになっている
            let unixepoch: Vec<UnixEpoch> = unixepoch_data_vec[cursor..cursor+size_list_vec[i]].to_vec();
            self.map.insert(geohash, unixepoch);
            cursor += size_list_vec[i];
        }
    }
}

/* 
GeohashTableWithPeriodArray
*/

#[derive(Clone, Default, Debug)]
pub struct GeohashTableWithPeriodArray {
    map: HashMap<GeoHashKey, Vec<Period>>
}

impl GeohashTableWithPeriodArray {
    pub fn new() -> Self {
        GeohashTableWithPeriodArray {
            map: HashMap::with_capacity(10000)
        }
    }

    pub fn intersect(&self, mapped_query_buffer: &MappedQueryBuffer, result: &mut ResultBuffer) {
        for (query_geohash, query_unixepoch_vec) in mapped_query_buffer.map.iter() {
            match self.map.get(query_geohash) {
                Some(dict_period_vec) => {
                    Self::judge_contact(query_unixepoch_vec, dict_period_vec, query_geohash, result);
                },
                None => {}
            }
        }

        // for (dict_geohash, dict_period_vec) in self.map.iter() {
        //     match mapped_query_buffer.map.get(dict_geohash) {
        //         Some(query_unixepoch_vec) => {
        //             Self::judge_contact(query_unixepoch_vec, dict_period_vec, dict_geohash, result);
        //         },
        //         None => {}
        //     }
        // }
    }

    /* CONTACT_TIME_THREASHOLDの幅で接触を判定して結果をResultBufferに返す */
    /* resultbufferにいれるところまでやるのは分かりにくいけど，パフォーマンス的にそうする */
    fn judge_contact(query_unixepoch_vec: &Vec<UnixEpoch>, dict_period_vec: &Vec<Period>, geohash: &GeoHashKey, result: &mut ResultBuffer) {
        let mut period_cursor = 0;
        let last = dict_period_vec.len();
        let finish = false;

        for query_unixepoch in query_unixepoch_vec.iter() {
            while period_cursor < last {
                if dict_period_vec[period_cursor].is_include(*query_unixepoch) {
                    result.data.push((*geohash, *query_unixepoch));
                    break;
                } else if *query_unixepoch < dict_period_vec[period_cursor].0 - CONTACT_TIME_THREASHOLD {
                    break;
                } else {
                    period_cursor += 1;
                }
            }
            if period_cursor == last { return; }
        }
    }

    fn build_dictionary_buffer(
        &mut self,
        geohash_data_vec: &Vec<u8>,
        period_data_vec: &Vec<u64>,
        size_list_vec: &Vec<usize>,
    ) {
        let mut cursor: usize = 0;
        for i in 0usize..(size_list_vec.len()) {
            let mut geohash = GeoHashKey::default(); 
            geohash.copy_from_slice(&geohash_data_vec[GEOHASH_U8_SIZE*i..GEOHASH_U8_SIZE*(i+1)]);
            let mut period: Vec<Period> = Vec::with_capacity(size_list_vec[i]);
            for j in 0..size_list_vec[i] {
                period.push(Period(period_data_vec[cursor+j*2], period_data_vec[cursor+j*2+1]));
            }
            self.map.insert(geohash, period);
            cursor += size_list_vec[i]*2;
        }
    }
}

/* Type QueryBuffer [q_1,...q_{N_c}]
    このデータ構造は基本的に固定して良いはず
*/

#[derive(Clone, Default, Debug)]
pub struct QueryBuffer {
    pub queries: Vec<QueryRep>,
}

impl QueryBuffer {
    pub fn new() -> Self {
        QueryBuffer::default()
    }

    // !!このメソッドでは全くerror処理していない
    // queryを個々に組み立ててbufferに保持する
    pub fn build_query_buffer(
        &mut self,
        total_query_data_vec: &Vec<u8>,
        size_list_vec       : &Vec<usize>,
        query_id_list_vec   : &Vec<u64>,
    ) -> i8 {
        let mut cursor = 0;
        for i in 0_usize..(size_list_vec.len()) {
            let size: usize = size_list_vec[i]*QUERY_U8_SIZE;
            let this_query = &total_query_data_vec[cursor..cursor+size];
            cursor = cursor+size; // 忘れないようにここで更新
            
            let mut query = QueryRep::new();
            query.id = query_id_list_vec[i];
            for i in 0_usize..(size/QUERY_U8_SIZE) {
                let mut timestamp = [0_u8; UNIXEPOCH_U8_SIZE];
                let mut geo_hash = [0_u8; GEOHASH_U8_SIZE];
                timestamp.copy_from_slice(&this_query[i*QUERY_U8_SIZE..i*QUERY_U8_SIZE+UNIXEPOCH_U8_SIZE]);
                geo_hash.copy_from_slice(&this_query[i*QUERY_U8_SIZE+UNIXEPOCH_U8_SIZE..(i + 1)*QUERY_U8_SIZE]);
                match query.parameters.get_mut(&geo_hash) {
                    Some(sorted_list) => { _sorted_push(sorted_list, unixepoch_from_u8(timestamp)) },
                    None => { query.parameters.insert(geo_hash as GeoHashKey, vec![unixepoch_from_u8(timestamp)]); },
                };
            }
            self.queries.push(query);
        }
        return 0;
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

/* ######################################################## */
/* Result */

/* 
Type QueryResult 
    バイトへのシリアライズを担当するよ
*/
#[derive(Clone, Default, Debug)]
pub struct QueryResult {
    pub query_id: QueryId,
    pub risk_level: u8,
}

impl QueryResult {
    pub fn new() -> Self {
        return QueryResult {
            query_id: 1,
            risk_level: 0,
        }
    }

    pub fn to_be_bytes(&self) -> [u8; RESPONSE_DATA_SIZE_U8] {
        let mut res = [0; RESPONSE_DATA_SIZE_U8];
        res[..QUERY_ID_SIZE_U8].clone_from_slice(&self.query_id.to_be_bytes());
        res[RESPONSE_DATA_SIZE_U8-QUERY_RESULT_U8] = self.risk_level;
        res
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

    // matchがネストして読みにくくなってしまっている
    // メソッドチェーンでもっと関数型っぽく書けば読みやすくなりそうではある
    pub fn build_query_response(
        &self,
        query_buffer: &QueryBuffer,
        response_vec: &mut Vec<u8>,
    ) {
        for query in query_buffer.queries.iter() {
            let mut result = QueryResult::new();
            result.query_id = query.id;
            for (geohash, unixepoch) in self.data.iter() {
                let is_exist = match query.parameters.get(geohash) {
                    Some(sorted_list) => { 
                        match sorted_list.binary_search(unixepoch) {
                            Ok(_) => true,
                            Err(_) => false,
                        }
                    },
                    None => { false },
                };
                if is_exist {
                    result.risk_level = 1;
                    break;
                }
            }
            response_vec.extend_from_slice(&result.to_be_bytes());
        }
    }
}

/* ######################################################## */
/* Private Method */

// なんかめっちゃ長くなってしまったけど
pub fn _sorted_merge(sorted_list_1: &Vec<UnixEpoch>, sorted_list_2: &Vec<UnixEpoch>) -> Vec<UnixEpoch> {
    let len1 = sorted_list_1.len();
    let len2 = sorted_list_2.len();
    let size = len1 + len2;
    let mut cursor1 = 0;
    let mut cursor2 = 0;
    let mut tmp_max = 0;

    let mut merged_vec = Vec::with_capacity(size);
    for _ in 0..size {
        let mut candidate = if sorted_list_1[cursor1] < sorted_list_2[cursor2] {
            cursor1 += 1;
            sorted_list_1[cursor1 - 1]
        } else if sorted_list_1[cursor1] == sorted_list_2[cursor2] {
            cursor1 += 1;
            cursor2 += 1;
            sorted_list_1[cursor1 - 1]
        } else {
            cursor2 += 1;
            sorted_list_2[cursor2 - 1]
        };
        
        if tmp_max != candidate { 
            tmp_max = candidate; 
            merged_vec.push(tmp_max);
        }
        
        if len1 == cursor1 && cursor2 < len2 {
            for j in cursor2..len2 {
                candidate = sorted_list_2[j];
                if tmp_max != candidate { tmp_max = candidate; } else { continue; };
                merged_vec.push(candidate);
            }
            break;
        }
        if len2 == cursor2 && cursor1 < len1 {
            for j in cursor1..len1 {
                candidate = sorted_list_1[j];
                if tmp_max != candidate { tmp_max = candidate; } else { continue; };
                merged_vec.push(candidate);
            }
            break;
        }
        if len1 == cursor1 && len2 == cursor2 { break; }
    }
    merged_vec
}

// 昇順ソート+ユニーク性
// あえてジェネリクスにする必要はない，むしろ型で守っていく
// Vecだと遅いけどLinkedListよりはキャッシュに乗るので早い気がするのでVecでいく
pub fn _sorted_push(sorted_list: &mut Vec<UnixEpoch>, unixepoch: UnixEpoch) {
    let mut index = 0;
    for elm in sorted_list.iter() {
        if *elm > unixepoch {
            sorted_list.insert(index, unixepoch);
            return;
        } else if *elm == unixepoch {
            return;
        } else {
            index += 1;
        }
    }
    sorted_list.push(unixepoch);
}

/* Type Period */
#[derive(Clone, Default, Debug)]
pub struct Period(UnixEpoch, UnixEpoch);

impl Period {
    pub fn new() -> Self {
        Period::default()
    }

    pub fn with_start(start: UnixEpoch) -> Self {
        Period(start, start)
    }

    pub fn from_unixepoch_vector(unixepoch_vec: &Vec<UnixEpoch>) -> Vec<Period> {
        let mut period_vec: Vec<Period> = vec![];
        
        assert!(unixepoch_vec.len() > 0);
        let mut latest_unixepoch: UnixEpoch = unixepoch_vec[0];
        let mut period = Period::with_start(latest_unixepoch);
        
        for unixepoch in unixepoch_vec.iter() {
            if latest_unixepoch + TIME_INTERVAL >= *unixepoch {
                latest_unixepoch = *unixepoch;
            } else {
                period.1 = latest_unixepoch;
                period_vec.push(period);
                period = Period::with_start(*unixepoch);
                latest_unixepoch = *unixepoch;
            }
        }
        period.1 = latest_unixepoch;
        period_vec.push(period);
        period_vec
    }

    // period - CONTACT_TIME_THREASHOLD < unixepoch < period + CONTACT_TIME_THREASHOLD
    pub fn is_include(&self, unixepoch: UnixEpoch) -> bool {
        self.0 - CONTACT_TIME_THREASHOLD < unixepoch && unixepoch < self.1 + CONTACT_TIME_THREASHOLD
    }
}