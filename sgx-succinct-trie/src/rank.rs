use std::vec::Vec;
use core::intrinsics::size_of;
use core::intrinsics::size_of_val;

use crate::config::*;
use crate::bitvector::BitVector;
use crate::popcount::{popcount_linear};

pub struct BitvectorRank {
    bitvec: BitVector,
    basic_block_size: position_t,
    rank_lut: Vec<position_t>,
}

impl BitvectorRank {
    pub fn serialize(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::with_capacity(1000);

        let bitvec_bytes = self.bitvec.serialize();
        bytes.extend(bitvec_bytes.len().to_be_bytes().iter());
        bytes.extend(bitvec_bytes);

        bytes.extend(self.basic_block_size.to_be_bytes().iter());

        bytes.extend(self.rank_lut.len().to_be_bytes().iter());
        for bit in self.rank_lut.iter() {
            bytes.extend(bit.to_be_bytes().iter());
        }
        bytes.shrink_to_fit();
        bytes
    }

    pub fn deserialize(bytes: &[u8]) -> Self {
        let mut cursor = 0;

        let mut bitvec_bytes_len_bytes: [u8; USIZE_BYTE_SIZE] = Default::default();
        bitvec_bytes_len_bytes.copy_from_slice(&bytes[cursor..cursor+USIZE_BYTE_SIZE]);
        cursor += USIZE_BYTE_SIZE;
        let bitvec_bytes_len = usize::from_be_bytes(bitvec_bytes_len_bytes);
        let bitvec = BitVector::deserialize(&bytes[cursor..cursor+bitvec_bytes_len]);
        cursor += bitvec_bytes_len;

        let mut basic_block_size_bytes: [u8; POSITION_T_BYTE_SIZE] = Default::default();
        basic_block_size_bytes.copy_from_slice(&bytes[cursor..cursor+POSITION_T_BYTE_SIZE]);
        cursor += POSITION_T_BYTE_SIZE;
        let basic_block_size = position_t::from_be_bytes(basic_block_size_bytes);

        let mut rank_lut_len_bytes: [u8; USIZE_BYTE_SIZE] = Default::default();
        rank_lut_len_bytes.copy_from_slice(&bytes[cursor..cursor+USIZE_BYTE_SIZE]);
        cursor += USIZE_BYTE_SIZE;
        let rank_lut_len = usize::from_be_bytes(rank_lut_len_bytes);

        let mut rank_lut: Vec<position_t> = Vec::with_capacity(rank_lut_len);
        for _ in 0..rank_lut_len {
            let mut bits_word_bytes: [u8; POSITION_T_BYTE_SIZE] = Default::default();
            bits_word_bytes.copy_from_slice(&bytes[cursor..cursor+POSITION_T_BYTE_SIZE]);
            cursor += POSITION_T_BYTE_SIZE;
            let pos = position_t::from_be_bytes(bits_word_bytes);
            rank_lut.push(pos);
        }

        BitvectorRank { bitvec, basic_block_size, rank_lut }
    }

    pub fn byte_size(&self) -> usize {
        let mut mem_size = 0;
        #[allow(unused_unsafe)]
        unsafe {
            mem_size += self.bitvec.byte_size();
            mem_size += size_of::<position_t>();
            mem_size += size_of_val(&*self.rank_lut);
        }
        mem_size
    }

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
