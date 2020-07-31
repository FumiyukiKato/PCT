use std::vec::Vec;
use std::collections::HashMap;

use primitive::*;
use constant::*;
use mapped_query_buffer::MappedQueryBuffer;
use result_buffer::ResultBuffer;

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

    pub fn build_dictionary_buffer(
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