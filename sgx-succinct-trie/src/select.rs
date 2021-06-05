use std::vec::Vec;
use core::intrinsics::{size_of, size_of_val};

use crate::bitvector::BitVector;
use crate::config::*;
use crate::popcount::select64_popcount_search;

pub struct BitvectorSelect {
    bitvec: BitVector,
    sample_interval: position_t,
    select_lut: Vec<position_t>,
}

impl BitvectorSelect {
    pub fn serialize(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::with_capacity(1000);

        let bitvec_bytes = self.bitvec.serialize();
        bytes.extend(bitvec_bytes.len().to_be_bytes().iter());
        bytes.extend(bitvec_bytes);

        bytes.extend(self.sample_interval.to_be_bytes().iter());

        bytes.extend(self.select_lut.len().to_be_bytes().iter());
        for bit in self.select_lut.iter() {
            bytes.extend(bit.to_be_bytes().iter());
        }
        bytes.shrink_to_fit();
        bytes
    }

    pub fn deserialize(bytes: &[u8]) -> Self {
        let mut cursor = 0;

        let mut bitvec_bytes_len_bytes: [u8; USIZE_BYTE_SIZE] = Default::default();
        bitvec_bytes_len_bytes.copy_from_slice(&bytes[cursor..cursor + USIZE_BYTE_SIZE]);
        cursor += USIZE_BYTE_SIZE;
        let bitvec_bytes_len = usize::from_be_bytes(bitvec_bytes_len_bytes);
        let bitvec = BitVector::deserialize(&bytes[cursor..cursor + bitvec_bytes_len]);
        cursor += bitvec_bytes_len;

        let mut sample_interval_bytes: [u8; POSITION_T_BYTE_SIZE] = Default::default();
        sample_interval_bytes.copy_from_slice(&bytes[cursor..cursor + POSITION_T_BYTE_SIZE]);
        cursor += POSITION_T_BYTE_SIZE;
        let sample_interval = position_t::from_be_bytes(sample_interval_bytes);

        let mut select_lut_len_bytes: [u8; USIZE_BYTE_SIZE] = Default::default();
        select_lut_len_bytes.copy_from_slice(&bytes[cursor..cursor + USIZE_BYTE_SIZE]);
        cursor += USIZE_BYTE_SIZE;
        let select_lut_len = usize::from_be_bytes(select_lut_len_bytes);

        let mut select_lut: Vec<position_t> = Vec::with_capacity(select_lut_len);
        for _ in 0..select_lut_len {
            let mut bits_word_bytes: [u8; POSITION_T_BYTE_SIZE] = Default::default();
            bits_word_bytes.copy_from_slice(&bytes[cursor..cursor + POSITION_T_BYTE_SIZE]);
            cursor += POSITION_T_BYTE_SIZE;
            let pos = position_t::from_be_bytes(bits_word_bytes);
            select_lut.push(pos);
        }

        BitvectorSelect {
            bitvec,
            sample_interval,
            select_lut,
        }
    }

    pub fn byte_size(&self) -> usize {
        let mut mem_size = 0;
        #[allow(unused_unsafe)]
        unsafe {
            mem_size += self.bitvec.byte_size();
            mem_size += size_of::<position_t>();
            mem_size += size_of_val(&*self.select_lut);
        }
        mem_size
    }

    pub fn new(
        sample_interval: position_t,
        bitvector_per_level: &Vec<Vec<word_t>>,
        num_bits_per_level: &Vec<position_t>,
        start_level: level_t,
        end_level: level_t,
    ) -> BitvectorSelect {
        let mut select = BitvectorSelect {
            bitvec: BitVector::new(
                bitvector_per_level,
                num_bits_per_level,
                start_level,
                end_level,
            ),
            sample_interval: sample_interval,
            select_lut: Vec::new(),
        };
        select.init_select_lut();
        select
    }

    fn init_select_lut(&mut self) {
        let mut num_words: position_t = self.bitvec.get_num_bits() / K_WORD_SIZE;
        if self.bitvec.get_num_bits() % K_WORD_SIZE != 0 {
            num_words += 1;
        }

        let mut select_lut_vector: Vec<position_t> = Vec::new();
        select_lut_vector.push(0); // ASSERT: first bit is 1

        let mut sampling_ones: position_t = self.sample_interval;
        let mut cumu_ones_upto_word: usize = 0;
        for i in 0..num_words {
            unsafe {
                let num_ones_in_word =
                    core::arch::x86_64::_popcnt64(self.bitvec.get_bits()[i] as i64) as u32;
                while sampling_ones <= cumu_ones_upto_word + num_ones_in_word as usize {
                    let diff: i32 = (sampling_ones - cumu_ones_upto_word) as i32;
                    let result_pos = i * K_WORD_SIZE
                        + select64_popcount_search(self.bitvec.get_bits()[i], diff) as usize;
                    select_lut_vector.push(result_pos);
                    sampling_ones += self.sample_interval;
                }
                cumu_ones_upto_word +=
                    core::arch::x86_64::_popcnt64(self.bitvec.get_bits()[i] as i64) as usize;
            }
        }
        self.select_lut = select_lut_vector;
    }

    pub fn get_bitvec(&self) -> &BitVector {
        &self.bitvec
    }

    // Returns the postion of the rank-th 1 bit.
    // posistion is zero-based; rank is one-based.
    // E.g., for bitvector: 100101000, select(3) = 5
    pub fn select(&self, rank: position_t) -> position_t {
        let lut_idx = rank / self.sample_interval;
        let mut rank_left = rank % self.sample_interval;
        // The first slot in select_lut_ stores the position of the first 1 bit.
        // Slot i > 0 stores the position of (i * sample_interval_)-th 1 bit
        if lut_idx == 0 {
            rank_left -= 1;
        }

        let pos = self.select_lut[lut_idx];

        if rank_left == 0 {
            return pos;
        }

        let mut word_id = pos / K_WORD_SIZE;
        let mut offset = pos % K_WORD_SIZE;
        if offset == K_WORD_SIZE - 1 {
            word_id += 1;
            offset = 0;
        } else {
            offset += 1;
        }
        let mut word: word_t = self.bitvec.get_bits()[word_id] << offset >> offset; // zero-out most significant bits
        unsafe {
            let mut ones_count_in_word: position_t =
                core::arch::x86_64::_popcnt64(word as i64) as position_t;
            while ones_count_in_word < rank_left {
                word_id += 1;
                word = self.bitvec.get_bits()[word_id];
                rank_left -= ones_count_in_word;
                ones_count_in_word = core::arch::x86_64::_popcnt64(word as i64) as position_t;
            }
        }
        return word_id * K_WORD_SIZE + select64_popcount_search(word, rank_left as i32) as usize;
    }
}
