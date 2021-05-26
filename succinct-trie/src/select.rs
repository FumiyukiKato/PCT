use crate::config::*;
use crate::bitvector::BitVector;
use crate::popcount::{select64_popcount_search};

pub struct BitvectorSelect {
    bitvec: BitVector,
    sample_interval: position_t,
    select_lut: Vec<position_t>,
}

impl BitvectorSelect {
    pub fn new (
        sample_interval: position_t,
        bitvector_per_level: &Vec<Vec<word_t>>,
        num_bits_per_level: &Vec<position_t>,
        start_level: level_t,
        end_level: level_t,
    ) -> BitvectorSelect {
        let mut select = BitvectorSelect {
            bitvec: BitVector::new(bitvector_per_level, num_bits_per_level, start_level, end_level),
            sample_interval: sample_interval,
            select_lut: Vec::new(),
        };
        select.init_select_lut();
        select
    }

    fn init_select_lut(&mut self) {
        let mut num_words: position_t = self.bitvec.get_num_bits() / K_WORD_SIZE;
        if self.bitvec.get_num_bits() % K_WORD_SIZE != 0 { num_words += 1; }

        let mut select_lut_vector: Vec<position_t> = Vec::new();
        select_lut_vector.push(0);  // ASSERT: first bit is 1

        let mut sampling_ones: position_t = self.sample_interval;
        let mut cumu_ones_upto_word: usize = 0;
        for i in 0..num_words {
            unsafe {
                let num_ones_in_word = core::arch::x86_64::_popcnt64(self.bitvec.get_bits()[i] as i64) as u32;
                while sampling_ones <= cumu_ones_upto_word + num_ones_in_word as usize {
                    let diff: i32 = (sampling_ones - cumu_ones_upto_word) as i32;
                    let result_pos = i * K_WORD_SIZE + select64_popcount_search(self.bitvec.get_bits()[i], diff) as usize;
                    select_lut_vector.push(result_pos);
                    sampling_ones += self.sample_interval;
                }
                cumu_ones_upto_word += core::arch::x86_64::_popcnt64(self.bitvec.get_bits()[i] as i64) as usize;
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
        if lut_idx == 0 { rank_left -= 1; }

        let pos = self.select_lut[lut_idx];

        if rank_left == 0 { return pos; }

        let mut word_id = pos / K_WORD_SIZE;
        let mut offset = pos % K_WORD_SIZE;
        if offset == K_WORD_SIZE - 1 {
            word_id += 1;
            offset = 0;
        } else {
            offset += 1;
        }
        let mut word: word_t = self.bitvec.get_bits()[word_id] << offset >> offset;  // zero-out most significant bits
        unsafe {
            let mut ones_count_in_word: position_t = core::arch::x86_64::_popcnt64(word as i64) as position_t;
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