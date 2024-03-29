use std::intrinsics::size_of;

use crate::builder::Builder;
// use crate::cache::Cache;
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
    pub fn serialize(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::with_capacity(1000);

        bytes.extend(self.height.to_be_bytes().iter());
        bytes.extend(self.start_level.to_be_bytes().iter());
        bytes.extend(self.node_count_dense.to_be_bytes().iter());
        bytes.extend(self.child_count_dense.to_be_bytes().iter());
        bytes.extend(self.value_count_dense.to_be_bytes().iter());

        let labels_bytes = self.labels.serialize();
        bytes.extend(labels_bytes.len().to_be_bytes().iter());
        bytes.extend(labels_bytes);

        let child_indicator_bits_bytes = self.child_indicator_bits.serialize();
        bytes.extend(child_indicator_bits_bytes.len().to_be_bytes().iter());
        bytes.extend(child_indicator_bits_bytes);

        let louds_bits_bytes = self.louds_bits.serialize();
        bytes.extend(louds_bits_bytes.len().to_be_bytes().iter());
        bytes.extend(louds_bits_bytes);

        bytes.shrink_to_fit();
        bytes
    }

    pub fn deserialize(bytes: &[u8]) -> Self {
        let mut cursor = 0;

        let mut height_bytes: [u8; LEVEL_T_BYTE_SIZE] = Default::default();
        height_bytes.copy_from_slice(&bytes[cursor..cursor + LEVEL_T_BYTE_SIZE]);
        cursor += LEVEL_T_BYTE_SIZE;
        let height = level_t::from_be_bytes(height_bytes);

        let mut start_level_bytes: [u8; LEVEL_T_BYTE_SIZE] = Default::default();
        start_level_bytes.copy_from_slice(&bytes[cursor..cursor + LEVEL_T_BYTE_SIZE]);
        cursor += LEVEL_T_BYTE_SIZE;
        let start_level = level_t::from_be_bytes(start_level_bytes);

        let mut node_count_dense_bytes: [u8; POSITION_T_BYTE_SIZE] = Default::default();
        node_count_dense_bytes.copy_from_slice(&bytes[cursor..cursor + POSITION_T_BYTE_SIZE]);
        cursor += POSITION_T_BYTE_SIZE;
        let node_count_dense = position_t::from_be_bytes(node_count_dense_bytes);

        let mut child_count_dense_bytes: [u8; POSITION_T_BYTE_SIZE] = Default::default();
        child_count_dense_bytes.copy_from_slice(&bytes[cursor..cursor + POSITION_T_BYTE_SIZE]);
        cursor += POSITION_T_BYTE_SIZE;
        let child_count_dense = position_t::from_be_bytes(child_count_dense_bytes);

        let mut value_count_dense_bytes: [u8; POSITION_T_BYTE_SIZE] = Default::default();
        value_count_dense_bytes.copy_from_slice(&bytes[cursor..cursor + POSITION_T_BYTE_SIZE]);
        cursor += POSITION_T_BYTE_SIZE;
        let value_count_dense = position_t::from_be_bytes(value_count_dense_bytes);

        let mut labels_len_bytes: [u8; USIZE_BYTE_SIZE] = Default::default();
        labels_len_bytes.copy_from_slice(&bytes[cursor..cursor + USIZE_BYTE_SIZE]);
        cursor += USIZE_BYTE_SIZE;
        let labels_len = usize::from_be_bytes(labels_len_bytes);
        let labels = LabelVector::deserialize(&bytes[cursor..cursor + labels_len]);
        cursor += labels_len;

        let mut child_indicator_bits_len_bytes: [u8; USIZE_BYTE_SIZE] = Default::default();
        child_indicator_bits_len_bytes.copy_from_slice(&bytes[cursor..cursor + USIZE_BYTE_SIZE]);
        cursor += USIZE_BYTE_SIZE;
        let child_indicator_bits_len = usize::from_be_bytes(child_indicator_bits_len_bytes);
        let child_indicator_bits =
            BitvectorRank::deserialize(&bytes[cursor..cursor + child_indicator_bits_len]);
        cursor += child_indicator_bits_len;

        let mut louds_bits_len_bytes: [u8; USIZE_BYTE_SIZE] = Default::default();
        louds_bits_len_bytes.copy_from_slice(&bytes[cursor..cursor + USIZE_BYTE_SIZE]);
        cursor += USIZE_BYTE_SIZE;
        let louds_bits_len = usize::from_be_bytes(louds_bits_len_bytes);
        let louds_bits = BitvectorSelect::deserialize(&bytes[cursor..cursor + louds_bits_len]);

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
    pub fn byte_size(&self) -> usize {
        let mut mem_size = 0;
        #[allow(unused_unsafe)]
        unsafe {
            mem_size += size_of::<level_t>() * 2 + size_of::<position_t>() * 3;
            mem_size += self.labels.byte_size();
            mem_size += self.child_indicator_bits.byte_size();
            mem_size += self.louds_bits.byte_size();
        }
        mem_size
    }

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

    pub fn find_key(&self, key: &key_t, in_node_num: level_t) -> (position_t, level_t) {
        let mut node_num = in_node_num;
        let mut pos = self.get_first_label_pos(node_num);

        for level in self.start_level..key.len() {
            self.child_indicator_bits.prefetch(pos);
            let (res, updated_pos) = self.labels.search(key[level], pos, self.node_size(pos));
            pos = updated_pos;
            if !res {
                return (K_NOT_FOUND, level);
            }
            if !self.child_indicator_bits.get_bitvec().read_bit(pos) {
                return (self.get_suffix_pos(pos) + self.value_count_dense, level + 1);
            }
            node_num = self.get_child_node_num(pos);
            pos = self.get_first_label_pos(node_num);
        }

        return (K_NOT_FOUND, key.len());
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
