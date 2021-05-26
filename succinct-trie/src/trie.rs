use crate::config::*;
use crate::louds_dense::LoudsDense;
use crate::louds_sparse::LoudsSparse;
use crate::builder;

pub struct Trie {
    louds_dense: LoudsDense,
    louds_sparse: LoudsSparse,
    suffixes: Vec<Suffix>,
}

// 生ポインタを使えばもっと速くなる
// ベクタofベクタだとキャッシュにも乗らない
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Suffix {
    contents: Vec<u8>,
}

impl Trie {
    pub fn new(keys: &Vec<Vec<u8>>) -> Self {
        let include_dense = K_INCLUDE_DENSE;
        let sparse_dense = K_SPARSE_DENSE_RATIO;

        let mut builder = builder::Builder::new(include_dense, sparse_dense);
        builder.build(&keys);
        let louds_dense = LoudsDense::new(&builder);
        let louds_sparse = LoudsSparse::new(&builder);

        let mut num_keys = 0;
        for level in 0..louds_sparse.get_height() {
            num_keys += builder.get_suffix_counts()[level];
        }

        let mut suffix_builder: Vec<Suffix> = vec![
            Suffix {
                contents: Vec::new(),
            };
            num_keys
        ];
        for i in 0..keys.len() {
            if i != 0 && keys[i] == keys[i - 1] {
                continue;
            }

            let (key_id, level) = Trie::traverse(&louds_dense, &louds_sparse, keys[i].as_slice());
            assert!(key_id < num_keys);
            let contents = keys[i][level..].to_vec();
            suffix_builder[key_id] = Suffix { contents };
        }
        // suffix_builder.sort();
        // let mut suffix_ptrs: Vec<usize> = vec![0; num_keys];
        // let mut suffixes = vec![];
        // let mut prev_suffix = Suffix {
        //     contents: Vec::new(),
        //     key_id: kNotFound,
        // };

        // for i in 0..num_keys {
        //     let curr_suffix = suffix_builder[num_keys - i - 1];
        //     if curr_suffix.contents.len() == 0 {
        //         suffix_ptrs[curr_suffix.key_id] = 0;
        //         continue;
        //     }
        //     let mut num_match = 0;
        //     while num_match < curr_suffix.contents.len()
        //         && num_match < prev_suffix.contents.len()
        //         && prev_suffix.contents[num_match] == curr_suffix.contents[num_match]
        //     {
        //         num_match += 1;
        //     }

        //     if num_match == curr_suffix.contents.len() && prev_suffix.contents.len() != 0 {
        //         suffix_ptrs[curr_suffix.key_id] = suffix_ptrs[prev_suffix.key_id] + (prev_suffix.contents.len() - num_match)
        //     } else {
        //         suffix_ptrs[curr_suffix.key_id] = suffixes.len();
        //         suffixes.push(curr_suffix);
        //     }
        //     prev_suffix = curr_suffix;
        // }

        // let mut suf_bits = 0;
        // let mut max_ptr = suffixes.len();

        // suf_bits += 1;
        // max_ptr >>= 1;
        // while max_ptr != 0 {
        //     suf_bits += 1;
        //     max_ptr >>= 1;
        // }
        // let suffix_ptrs = 

        return Trie {
            louds_dense,
            louds_sparse,
            suffixes: suffix_builder,
        }
    }

    fn traverse(
        louds_dense: &LoudsDense,
        louds_sparse: &LoudsSparse,
        key: &key_t,
    ) -> (position_t, level_t) {
        let ret = louds_dense.find_key(key);
        if ret.0 != K_NOT_FOUND {
            return (ret.0, ret.1);
        }
        if ret.2 != K_NOT_FOUND {
            return louds_sparse.find_key(key, ret.2);
        }
        return (ret.0, ret.1);
    }

    fn _traverse(
        &self,
        key: &key_t,
    ) -> (position_t, level_t) {
        let ret = self.louds_dense.find_key(key);
        if ret.0 != K_NOT_FOUND {
            return (ret.0, ret.1);
        }
        if ret.2 != K_NOT_FOUND {
            return self.louds_sparse.find_key(key, ret.2);
        }
        return (ret.0, ret.1);
    }

    pub fn exact_search(&self, key: &key_t) -> position_t {
        let (key_id, level) = self._traverse(key);
        if key_id == K_NOT_FOUND {
            return K_NOT_FOUND
        }

        let suffix = &self.suffixes[key_id].contents;
        let length = key.len() - level;
        if length != suffix.len() {
            return K_NOT_FOUND
        }

        for (cur_key, cur_suf) in key[level..].iter().zip(suffix.iter()) {
            if cur_key != cur_suf {
                return K_NOT_FOUND
            }
        }
        return key_id
    }

    // // 見つかったかどうか，直前の探索のログを返したい．
    // fn caching_search(&self, previous_key: &key_t, key: &key_t, cache: Cache) -> position_t {
    //     let diff_level = self.find_different_level(previous_key, key);
    //     let (key_id, level) = 
    //         if diff_level < self.louds_sparse.get_start_level() {
    //             let ret = self.louds_dense.find_key_with_cache(key, cache, diff_level);
    //             if ret.0 != K_NOT_FOUND {
    //                 (ret.0, ret.1)
    //             } else if ret.2 != K_NOT_FOUND {
    //                 self.louds_sparse.find_key_with_cache(key, ret.2, cache, diff_level)
    //             } else {
    //                 (ret.0, ret.1)
    //             }
    //         } else {
    //             self.louds_sparse.find_key_with_cache(key, 0, cache, diff_level)
    //         };
        
    // }

    // fn find_different_level(&self, pre_key: &key_t, key: &key_t) -> level_t {
    //     let mut diff_level = 0;
    //     for (p, k) in pre_key.iter().zip(key) {
    //         if p != k {
    //             return diff_level
    //         } else {
    //             diff_level += 1;
    //         }
    //     }
    //     return diff_level
    // }

    // time_range is depends on encoding specification
    pub fn doe_search(&self, time_range: usize, keys: &Vec<Vec<u8>>) -> bool {
        let mut sequnce_count = 0;
        let th = TrajectoryHash::new(7, 20, 16);
        for key in keys.iter() {
            // let result = self.exact_search(&key);
            // let is_find = result != K_NOT_FOUND;
            let is_find = self.accurate_search(key, &th);
            if is_find {
                sequnce_count += 1;
                if sequnce_count >= time_range {
                    return true
                }
            } else {
                sequnce_count = 0;
            }
        }
        return false
    }

    pub fn accurate_search(&self, key: &key_t, th: &TrajectoryHash) -> bool {
        let neighbors = self.get_neighbors(key, th);
        for nei in neighbors {
            if self.exact_search(nei.as_slice()) != K_NOT_FOUND {
                return true
            }
        }
        false
    }

    pub fn get_neighbors(&self, key: &key_t, th: &TrajectoryHash) -> Vec<Vec<u8>> {

        let mut vec = Vec::with_capacity(EXTEND_NUMBER);
        let value: u128 = read_be_u128(key);

        // tiles to hash values
        for position in ACCURATE_GRID {
            let bytes = u128_to_bytes(th.calc(value, position), th.byte_length);
            vec.push(bytes);
        }  
        vec

    }
}

pub struct TrajectoryHash {
    byte_length: usize,
    pub mask_lists: [Vec<u128>; 3], // ascend order
}

impl TrajectoryHash {
    pub fn new(byte_length: usize, geo_length: usize, time_length: usize) -> Self {
        let mut geo_lng_mask = 0b100u128;
        let mut geo_lat_mask = 0b010u128;
        let mut time_mask    = 0b001u128;

        let diff = (geo_length as i32) - (time_length as i32);
        let mut mask_lists = [Vec::new(), Vec::new(), Vec::new()];
        if diff >= 0 {
            for _ in 0..time_length {
                mask_lists[0].push(geo_lng_mask);
                geo_lng_mask <<= 3;
                mask_lists[1].push(geo_lat_mask);
                geo_lat_mask <<= 3;
                mask_lists[2].push(time_mask);
                time_mask <<= 3;
            }
            geo_lng_mask >>= 3;
            geo_lng_mask <<= 2;
            geo_lat_mask >>= 3;
            geo_lat_mask <<= 2;
            for _ in 0..diff {
                mask_lists[0].push(geo_lng_mask);
                geo_lng_mask <<= 2;
                mask_lists[1].push(geo_lat_mask);
                geo_lat_mask <<= 2;
            }
        } else {
            for _ in 0..geo_length {
                mask_lists[0].push(geo_lng_mask);
                geo_lng_mask <<= 3;
                mask_lists[1].push(geo_lat_mask);
                geo_lat_mask <<= 3;
                mask_lists[2].push(time_mask);
                time_mask <<= 3;
            }
            for _ in 0..(-diff) {
                mask_lists[2].push(time_mask);
                time_mask <<= 1;
            }
        }

        TrajectoryHash { byte_length, mask_lists }
    }

    pub fn calc(&self, value: u128, pos: [i32;3]) -> u128 {
        let mut updated = value;
        for (dimension, direction) in pos.iter().enumerate() {
            match direction {
                -1 => {
                    for mask in self.mask_lists[dimension].iter() {
                        if value & mask != 0 {
                            updated &= !mask;
                            break;
                        } else {
                            updated |= mask;
                        }
                    }
                },
                0 => {},
                1 => {
                    for mask in self.mask_lists[dimension].iter() {
                        if value & mask == 0 {
                            updated |= mask;
                            break;
                        } else {
                            updated &= !mask;
                        }
                    }
                },
                _ => panic!("invalid value of direction!")
            }
        }
        updated
    }
}

fn read_be_u128(input: &[u8]) -> u128 {
    let mut output = 0u128;
    let digit = input.len() - 1;
    for (i, byte) in input.iter().enumerate() {
        output |= (*byte as u128) << 8*(digit - i);
    }
    output
}

fn u128_to_bytes(value: u128, byte_length: usize) -> Vec<u8> {
    value.to_be_bytes()[16-byte_length..].to_vec()
}