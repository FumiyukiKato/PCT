use std::vec::Vec;
use std::collections::HashMap;

use primitive::*;
use constant::*;
use period::*;
use mapped_query_buffer::MappedQueryBuffer;
use result_buffer::ResultBuffer;
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
                } else if *query_unixepoch < dict_period_vec[period_cursor].start() - CONTACT_TIME_THREASHOLD {
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
                period.push(Period::new(period_data_vec[cursor+j*2], period_data_vec[cursor+j*2+1]));
            }
            self.map.insert(geohash, period);
            cursor += size_list_vec[i]*2;
        }
    }
}
