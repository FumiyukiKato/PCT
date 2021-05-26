use crate::builder::Builder;
use crate::cache::Cache;
use crate::config::*;
use crate::label_vector::LabelVector;
use crate::rank::BitvectorRank;
use crate::select::BitvectorSelect;
use crate::suffix::BitvectorSuffix;

const K_RANK_BASIC_BLOCK_SIZE: position_t = 512;
const K_SELECT_SAMPLE_INTERVAL: position_t = 64;

pub struct LoudsSparse {
    // Modified by Shunsuke Kanda
    height: level_t,      // trie height
    start_level: level_t, // louds-sparse encoding starts at this level
    // number of nodes in louds-dense encoding
    node_count_dense: position_t,
    // number of children(1's in child indicator bitmap) in louds-dense encoding
    child_count_dense: position_t,

    value_count_dense: position_t,

    labels: LabelVector,
    child_indicator_bits: BitvectorRank,
    louds_bits: BitvectorSelect,
}

impl LoudsSparse {
    pub fn new(builder: &Builder) -> LoudsSparse {
        let height = builder.get_labels().len();
        let start_level = builder.get_sparse_start_level();
        let mut node_count_dense = 0;

        for level in 0..start_level {
            node_count_dense += builder.get_node_counts()[level];
        }

        let child_count_dense = if start_level == 0 {
            0
        } else {
            node_count_dense + builder.get_node_counts()[start_level] - 1
        };

        let labels = LabelVector::new(builder.get_labels(), start_level, height);

        let mut num_items_per_level: Vec<position_t> = Vec::new();
        for level in 0..height {
            num_items_per_level.push(builder.get_labels()[level].len());
        }

        let child_indicator_bits = BitvectorRank::new(
            K_RANK_BASIC_BLOCK_SIZE,
            builder.get_child_indicator_bits(),
            &num_items_per_level,
            start_level,
            height,
        );
        let louds_bits = BitvectorSelect::new(
            K_SELECT_SAMPLE_INTERVAL,
            builder.get_louds_bits(),
            &num_items_per_level,
            start_level,
            height,
        );

        let (_, value_count_dense) = match builder.get_suffix_type() {
            _ => {
                let suffixes = BitvectorSuffix::new();
                let mut value_count_dense = 0;
                for level in 0..start_level {
                    value_count_dense += builder.get_suffix_counts()[level]
                }
                (suffixes, value_count_dense)
            }
        };

        LoudsSparse {
            height,
            start_level,
            node_count_dense,
            child_count_dense,
            value_count_dense,
            labels,
            child_indicator_bits,
            louds_bits,
        }
    }

    pub fn get_height(&self) -> level_t {
        self.height
    }

    pub fn get_start_level(&self) -> level_t {
        self.start_level
    }

    pub fn find_key(&self, key: &key_t, in_node_num: level_t) -> (position_t, level_t) {
        let mut node_num = in_node_num;
        let mut pos = self.get_first_label_pos(node_num);

        for level in self.start_level..key.len() {
            self.child_indicator_bits.prefetch(pos);
            let (res, updated_pos) = self.labels.search(key[level], pos, self.node_size(pos));
            pos = updated_pos;
            if !res { 
                return (K_NOT_FOUND, level) 
            }
            if !self.child_indicator_bits.get_bitvec().read_bit(pos) {
                return (self.get_suffix_pos(pos) + self.value_count_dense, level + 1)
            }
            node_num = self.get_child_node_num(pos);
            pos = self.get_first_label_pos(node_num);
        }

        if self.labels.read(pos) == K_TERMINATOR && !self.child_indicator_bits.get_bitvec().read_bit(pos) {
            return (self.get_suffix_pos(pos) + self.value_count_dense, key.len())
        }
        return (K_NOT_FOUND, key.len())
    }

    // pub fn find_key_with_cache(&self, key: &key_t, in_node_num: level_t, cache: Cache, diff_level: level_t) -> (position_t, level_t) {
    //     let mut node_num = in_node_num;
    //     let mut pos = self.get_first_label_pos(node_num);

    //     for level in self.start_level..key.len() {
    //         self.child_indicator_bits.prefetch(pos);
    //         let (res, updated_pos) = self.labels.search(key[level], pos, self.node_size(pos));
    //         pos = updated_pos;
    //         if !res { 
    //             return (K_NOT_FOUND, level) 
    //         }
    //         if !self.child_indicator_bits.get_bitvec().read_bit(pos) {
    //             return (self.get_suffix_pos(pos) + self.value_count_dense, level + 1)
    //         }
    //         node_num = self.get_child_node_num(pos);
    //         pos = self.get_first_label_pos(node_num);
    //     }

    //     if self.labels.read(pos) == K_TERMINATOR && !self.child_indicator_bits.get_bitvec().read_bit(pos) {
    //         return (self.get_suffix_pos(pos) + self.value_count_dense, key.len())
    //     }
    //     return (K_NOT_FOUND, key.len())
    // }

    fn get_first_label_pos(&self, node_num: position_t) -> position_t {
        self.louds_bits.select(node_num + 1 - self.node_count_dense)
    }

    fn node_size(&self, pos: position_t) -> position_t {
        self.louds_bits.get_bitvec().distance_to_next_set_bit(pos)
    }

    fn get_suffix_pos(&self, pos: position_t) -> position_t {
        pos - self.child_indicator_bits.rank(pos)
    }

    fn get_child_node_num(&self, pos: position_t) -> position_t {
        self.child_indicator_bits.rank(pos) + self.child_count_dense
    }
}


