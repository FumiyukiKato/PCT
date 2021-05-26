pub const K_INCLUDE_DENSE: bool = true;
pub const K_SPARSE_DENSE_RATIO: u32 = 64;
pub const K_WORD_SIZE: usize = 64;

#[allow(non_camel_case_types)]
pub type level_t = usize;
#[allow(non_camel_case_types)]
pub type position_t = usize;
#[allow(non_camel_case_types)]
pub type word_t = u64;
#[allow(non_camel_case_types)]
pub type label_t = u8;
#[allow(non_camel_case_types)]
pub type node_t = u8;
#[allow(non_camel_case_types)]
pub type key_t = [node_t];


pub const K_MSB_MASK: word_t = 0x8000000000000000; // 1000000000000000000000000000000000000000000000000000000000000000
pub const K_ONE_MASK: word_t = 0xFFFFFFFFFFFFFFFF;

// TODO; これバグらない？kTerminatorで最後か判断しているところがあれば．．．
pub const K_TERMINATOR: label_t = 0;

#[derive(Copy, Clone)]
pub enum SuffixType {
    KNone
}

pub const K_FANOUT: position_t = 256;
// TODO; これなんとかしたい．別に大丈夫？
pub const K_NOT_FOUND: position_t = usize::MAX;

pub const EXTEND_NUMBER: usize = 27;

pub const ACCURATE_GRID: [[i32; 3]; EXTEND_NUMBER] = [
    [-1, -1, -1],
    [-1, -1, 0],
    [-1, -1, 1],
    [-1, 0, -1],
    [-1, 0, 0],
    [-1, 0, 1],
    [-1, 1, -1],
    [-1, 1, 0],
    [-1, 1, 1],
    [0, -1, -1],
    [0, -1, 0],
    [0, -1, 1],
    [0, 0, -1],
    [0, 0, 0],
    [0, 0, 1],
    [0, 1, -1],
    [0, 1, 0],
    [0, 1, 1],
    [1, -1, -1],
    [1, -1, 0],
    [1, -1, 1],
    [1, 0, -1],
    [1, 0, 0],
    [1, 0, 1],
    [1, 1, -1],
    [1, 1, 0],
    [1, 1, 1],
];