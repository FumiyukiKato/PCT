use crate::builder::Builder;
use crate::cache::Cache;
use crate::config::*;
use crate::rank::BitvectorRank;

const K_NODE_FANOUT: position_t = 256;
const K_RANK_BASIC_BLOCK_SIZE: position_t = 512;

pub struct LoudsDense {
    height: level_t,
    label_bitmaps: BitvectorRank,
    child_indicator_bitmaps: BitvectorRank,
    prefixkey_indicator_bits: BitvectorRank,
}

impl LoudsDense {
    pub fn new(builder: &Builder) -> LoudsDense {
        let height = builder.get_sparse_start_level();
        let mut num_bits_per_level: Vec<position_t> = Vec::new();
        for level in 0..height {
            num_bits_per_level.push(builder.get_bit_map_labels()[level].len() * K_WORD_SIZE);
        }
        let label_bitmaps = BitvectorRank::new(
            K_RANK_BASIC_BLOCK_SIZE,
            builder.get_bit_map_labels(),
            &num_bits_per_level,
            0,
            height,
        );
        let child_indicator_bitmaps = BitvectorRank::new(
            K_RANK_BASIC_BLOCK_SIZE,
            builder.get_bitmap_child_indicator_bits(),
            &num_bits_per_level,
            0,
            height,
        );
        let prefixkey_indicator_bits = BitvectorRank::new(
            K_RANK_BASIC_BLOCK_SIZE,
            builder.get_prefixkey_indicator_bits(),
            &builder.get_node_counts(),
            0,
            height,
        );

        LoudsDense {
            height: height,
            label_bitmaps: label_bitmaps,
            child_indicator_bitmaps: child_indicator_bitmaps,
            prefixkey_indicator_bits: prefixkey_indicator_bits,
        }
    }

    pub fn find_key(&self, key: &key_t) -> (position_t, level_t, position_t) {
        let mut node_num: position_t = 0;
        let mut out_node_num = K_NOT_FOUND;

        for level in 0..self.height {
            let mut pos = node_num * K_NODE_FANOUT;
            if level >= key.len() {
                if self.prefixkey_indicator_bits.get_bitvec().read_bit(node_num) {
                    return (self.get_suffix_pos(pos, true), level, out_node_num)
                } else {
                    return (K_NOT_FOUND, level, out_node_num)
                }
            }
            pos += key[level] as level_t;
            self.child_indicator_bitmaps.prefetch(pos);
            
            if !self.label_bitmaps.get_bitvec().read_bit(pos) {
                return (K_NOT_FOUND, level + 1, out_node_num)
            }

            if !self.child_indicator_bitmaps.get_bitvec().read_bit(pos) {
                return (self.get_suffix_pos(pos, false), level + 1, out_node_num)
            }

            node_num = self.get_child_node_num(pos);
        }
        out_node_num = node_num;
        return (K_NOT_FOUND, self.height, out_node_num)
    }

    // pub fn find_key_with_cache(&self, key: &key_t, cache: Cache, diff_level: level_t) -> (position_t, level_t, position_t) {
    //     let mut node_num: position_t = cache.get_pos(diff_level);
    //     let mut out_node_num = cache.get_pos(diff_level);

    //     for level in diff_level..self.height {
    //         let mut pos = node_num * K_NODE_FANOUT;
    //         if level >= key.len() {
    //             if self.prefixkey_indicator_bits.get_bitvec().read_bit(node_num) {
    //                 return (self.get_suffix_pos(pos, true), level, out_node_num)
    //             } else {
    //                 return (K_NOT_FOUND, level, out_node_num)
    //             }
    //         }
    //         pos += key[level] as level_t;
    //         self.child_indicator_bitmaps.prefetch(pos);
            
    //         if !self.label_bitmaps.get_bitvec().read_bit(pos) {
    //             return (K_NOT_FOUND, level + 1, out_node_num)
    //         }

    //         if !self.child_indicator_bitmaps.get_bitvec().read_bit(pos) {
    //             return (self.get_suffix_pos(pos, false), level + 1, out_node_num)
    //         }

    //         node_num = self.get_child_node_num(pos);
    //     }
    //     out_node_num = node_num;
    //     return (K_NOT_FOUND, self.height, out_node_num)
    // }

    fn get_suffix_pos(&self, pos: position_t, is_prefix_key: bool) -> position_t {
        let node_num: position_t = pos / K_NODE_FANOUT;
        let mut suffix_pos: position_t = self.label_bitmaps.rank(pos) - self.child_indicator_bitmaps.rank(pos) + self.prefixkey_indicator_bits.rank(node_num) - 1;
        if is_prefix_key && self.label_bitmaps.get_bitvec().read_bit(pos) && !self.child_indicator_bitmaps.get_bitvec().read_bit(pos) {
            suffix_pos -= 1;
        }
        return suffix_pos;
    }

    fn get_child_node_num(&self, pos: position_t) -> position_t {
        self.child_indicator_bitmaps.rank(pos)
    }
}
