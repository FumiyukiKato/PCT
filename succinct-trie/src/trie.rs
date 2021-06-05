use core::intrinsics::size_of_val;

use crate::builder;
use crate::config::*;
use crate::louds_dense::LoudsDense;
use crate::louds_sparse::LoudsSparse;

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

impl Suffix {
    pub fn serialize(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::with_capacity(10);
        bytes.extend(self.contents.len().to_be_bytes().iter());
        bytes.extend(self.contents.as_slice());
        bytes
    }

    pub fn deserialize(bytes: &[u8]) -> Self {
        let mut cursor = 0;

        let mut suffix_len_bytes: [u8; USIZE_BYTE_SIZE] = Default::default();
        suffix_len_bytes.copy_from_slice(&bytes[cursor..cursor+USIZE_BYTE_SIZE]);
        cursor += USIZE_BYTE_SIZE;
        let suffix_len = usize::from_be_bytes(suffix_len_bytes);

        let mut contents: Vec<u8> = Vec::with_capacity(suffix_len);
        for _ in 0..suffix_len {
            let mut suffix_word_bytes: [u8; 1] = Default::default();
            suffix_word_bytes.copy_from_slice(&bytes[cursor..cursor+1]);
            cursor += 1;
            let word = u8::from_be_bytes(suffix_word_bytes);
            contents.push(word);
        }
        Suffix { contents }
    }
}

impl Trie {
    pub fn serialize(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::with_capacity(100000);

        let louds_dense_bytes = self.louds_dense.serialize();
        bytes.extend(louds_dense_bytes.len().to_be_bytes().iter());
        bytes.extend(louds_dense_bytes);

        let louds_sparse_bytes = self.louds_sparse.serialize();
        bytes.extend(louds_sparse_bytes.len().to_be_bytes().iter());
        bytes.extend(louds_sparse_bytes);

        bytes.extend(self.suffixes.len().to_be_bytes().iter());
        for suffix in self.suffixes.iter() {
            let suffix_bytes = suffix.serialize();
            bytes.extend(suffix_bytes.len().to_be_bytes().iter());
            bytes.extend(suffix_bytes);
        }
        bytes
    }

    pub fn deserialize(bytes: &[u8]) -> Self {
        let mut cursor = 0;

        let mut louds_dense_len_bytes: [u8; USIZE_BYTE_SIZE] = Default::default();
        louds_dense_len_bytes.copy_from_slice(&bytes[cursor..cursor+USIZE_BYTE_SIZE]);
        cursor += USIZE_BYTE_SIZE;
        let louds_dense_len = usize::from_be_bytes(louds_dense_len_bytes);
        let louds_dense = LoudsDense::deserialize(&bytes[cursor..cursor+louds_dense_len]);
        cursor += louds_dense_len;

        let mut louds_sparse_len_bytes: [u8; USIZE_BYTE_SIZE] = Default::default();
        louds_sparse_len_bytes.copy_from_slice(&bytes[cursor..cursor+USIZE_BYTE_SIZE]);
        cursor += USIZE_BYTE_SIZE;
        let louds_sparse_len = usize::from_be_bytes(louds_sparse_len_bytes);
        let louds_sparse = LoudsSparse::deserialize(&bytes[cursor..cursor+louds_sparse_len]);
        cursor += louds_sparse_len;

        let mut suffixes_len_bytes: [u8; USIZE_BYTE_SIZE] = Default::default();
        suffixes_len_bytes.copy_from_slice(&bytes[cursor..cursor+USIZE_BYTE_SIZE]);
        cursor += USIZE_BYTE_SIZE;
        let suffixes_len = usize::from_be_bytes(suffixes_len_bytes);
        println!("suffixes_len: {}", suffixes_len);

        let mut suffixes: Vec<Suffix> = Vec::with_capacity(suffixes_len);
        for _ in 0..suffixes_len {
            let mut suffix_len_bytes: [u8; USIZE_BYTE_SIZE] = Default::default();
            suffix_len_bytes.copy_from_slice(&bytes[cursor..cursor+USIZE_BYTE_SIZE]);
            cursor += USIZE_BYTE_SIZE;
            let suffix_len = usize::from_be_bytes(suffix_len_bytes);
            println!("suffix_len: {}", suffix_len);
            let suffix = Suffix::deserialize(&bytes[cursor..cursor+suffix_len]);
            cursor += suffix_len;
            suffixes.push(suffix);
        }

        Trie { louds_dense, louds_sparse, suffixes }
    }

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

            if !(key_id < num_keys) {
                println!("i {}", i);
                println!("key_id {}", key_id);
                println!("keys[i] {:?}", keys[i]);
            }
            assert!(key_id < num_keys);
            let contents = keys[i][level..].to_vec();
            suffix_builder[key_id] = Suffix { contents };
        }

        let mut suffix_len_vec = vec![0, 0, 0, 0 ,0,0,0];
        for s in suffix_builder.iter() {
            suffix_len_vec[s.contents.len()] += 1;
        }
        println!("suffix_len_vec : {:?}", suffix_len_vec);

        return Trie {
            louds_dense,
            louds_sparse,
            suffixes: suffix_builder,
        };
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

    fn _traverse(&self, key: &key_t) -> (position_t, level_t) {
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
            return K_NOT_FOUND;
        }

        let suffix = &self.suffixes[key_id].contents;
        let length = key.len() - level;
        if length != suffix.len() {
            return K_NOT_FOUND;
        }

        for (cur_key, cur_suf) in key[level..].iter().zip(suffix.iter()) {
            if cur_key != cur_suf {
                return K_NOT_FOUND;
            }
        }
        return key_id;
    }

    pub fn contains(&self, key: &key_t) -> bool {
        K_NOT_FOUND != self.exact_search(key)
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
        for key in keys.iter() {
            let result = self.exact_search(&key);
            let is_find = result != K_NOT_FOUND;
            if is_find {
                sequnce_count += 1;
                if sequnce_count >= time_range {
                    return true;
                }
            } else {
                sequnce_count = 0;
            }
        }
        return false;
    }

    // time_range is depends on encoding specification
    pub fn accurate_doe_search(
        &self,
        time_range: usize,
        keys: &Vec<Vec<u8>>,
        th: &TrajectoryHash,
    ) -> bool {
        let mut sequnce_count = 0;
        for key in keys.iter() {
            let is_find = self.accurate_search(key, &th);
            if is_find {
                sequnce_count += 1;
                if sequnce_count >= time_range {
                    return true;
                }
            } else {
                sequnce_count = 0;
            }
        }
        return false;
    }

    pub fn accurate_search(&self, key: &key_t, th: &TrajectoryHash) -> bool {
        let neighbors = th.get_neighbors(key);
        for nei in neighbors {
            if self.exact_search(nei.as_slice()) != K_NOT_FOUND {
                return true;
            }
        }
        false
    }

    pub fn byte_size(&self) -> usize {
        let mut mem_size = 0;
        #[allow(unused_unsafe)]
        unsafe {
            mem_size += size_of_val(&*self.suffixes);
            println!("suffix: {}", size_of_val(&*self.suffixes));
            mem_size += self.louds_dense.byte_size();
            println!("louds_dense: {}", self.louds_dense.byte_size());
            mem_size += self.louds_sparse.byte_size();
            println!("louds_sparse: {}", self.louds_sparse.byte_size());
        }
        mem_size
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
        let mut time_mask = 0b001u128;

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

        TrajectoryHash {
            byte_length,
            mask_lists,
        }
    }

    pub fn get_neighbors(&self, key: &key_t) -> Vec<Vec<u8>> {
        let mut vec = Vec::with_capacity(EXTEND_NUMBER);
        let value: u128 = read_be_u128(key);

        // tiles to hash values
        for position in ACCURATE_GRID.iter() {
            let bytes = u128_to_bytes(self.calc(value, *position), self.byte_length);
            vec.push(bytes);
        }
        vec
    }

    pub fn calc(&self, value: u128, pos: [i32; 3]) -> u128 {
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
                }
                0 => {}
                1 => {
                    for mask in self.mask_lists[dimension].iter() {
                        if value & mask == 0 {
                            updated |= mask;
                            break;
                        } else {
                            updated &= !mask;
                        }
                    }
                }
                _ => panic!("invalid value of direction!"),
            }
        }
        updated
    }
}

fn read_be_u128(input: &[u8]) -> u128 {
    let mut output = 0u128;
    let digit = input.len() - 1;
    for (i, byte) in input.iter().enumerate() {
        output |= (*byte as u128) << 8 * (digit - i);
    }
    output
}

fn u128_to_bytes(value: u128, byte_length: usize) -> Vec<u8> {
    value.to_be_bytes()[16 - byte_length..].to_vec()
}
