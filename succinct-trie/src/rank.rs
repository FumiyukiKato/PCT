use crate::config::*;
use crate::bitvector::BitVector;
use crate::popcount::{popcount_linear};

pub struct BitvectorRank {
    bitvec: BitVector,
    basic_block_size: position_t,
    rank_lut: Vec<position_t>,
}

impl BitvectorRank {
    pub fn new(
        basic_block_size: position_t,
        bitvector_per_level: &Vec<Vec<word_t>>,
        num_bits_per_level: &Vec<position_t>,
        start_level: level_t,
        end_level: level_t,
    ) -> BitvectorRank {
        let mut rank = BitvectorRank {
            bitvec: BitVector::new(bitvector_per_level, num_bits_per_level, start_level, end_level),
            basic_block_size: basic_block_size,
            rank_lut: Vec::new(),
        };
        rank.init_rank_lut();
        rank
    }

    fn init_rank_lut(&mut self) {
        let word_per_basic_block: position_t = self.basic_block_size / K_WORD_SIZE;
        let num_blocks: position_t = self.bitvec.get_num_bits() / self.basic_block_size + 1;

        let mut rank_lut = vec![0; num_blocks];
        let mut cumu_rank: position_t = 0;

        for i in 0..(num_blocks-1) {
            rank_lut[i] = cumu_rank;
            cumu_rank += popcount_linear(self.bitvec.get_bits(), (i * word_per_basic_block) as u64, self.basic_block_size as u64) as usize;
        }
        rank_lut[num_blocks - 1] = cumu_rank;
        self.rank_lut = rank_lut;
    }

    pub fn get_bitvec(&self) -> &BitVector {
        &self.bitvec
    }

    // Counts the number of 1's in the bitvector up to position pos.
    // pos is zero-based; count is one-based.
    // E.g., for bitvector: 100101000, rank(3) = 2
    pub fn rank(&self, pos: position_t) -> position_t {
        let word_per_basic_block: position_t = self.basic_block_size / K_WORD_SIZE;
        let block_id: position_t = pos / self.basic_block_size;
        let offset: position_t = pos & (self.basic_block_size - 1);
        return self.rank_lut[block_id] + popcount_linear(self.bitvec.get_bits(), (block_id * word_per_basic_block) as u64, (offset + 1) as u64) as position_t;
    }

    pub fn prefetch(&self, pos: position_t) {
        unsafe {
            let bits_pointer = self.bitvec.get_bits().as_ptr();
            core::intrinsics::prefetch_read_data(bits_pointer.offset((pos / K_WORD_SIZE) as isize), 0);
            let rank_lut_pointer = self.rank_lut.as_ptr();
            core::intrinsics::prefetch_read_data(rank_lut_pointer.offset((pos / self.basic_block_size) as isize), 0);
        }
    }
}
