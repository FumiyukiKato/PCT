use core::intrinsics::{size_of_val, size_of};

use crate::config::*;

pub struct BitVector {
    num_bits: position_t,
    bits: Vec<word_t>,
}

impl BitVector{
    pub fn serialize(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::with_capacity(1000);
        bytes.extend(self.num_bits.to_be_bytes().iter());
        bytes.extend(self.bits.len().to_be_bytes().iter());
        for bit in self.bits.iter() {
            bytes.extend(bit.to_be_bytes().iter());
        }
        bytes.shrink_to_fit();
        bytes
    }

    pub fn deserialize(bytes: &[u8]) -> Self {
        let mut cursor = 0;
        let mut num_bits_byte: [u8; POSITION_T_BYTE_SIZE] = Default::default();
        num_bits_byte.copy_from_slice(&bytes[cursor..cursor+POSITION_T_BYTE_SIZE]);
        cursor += POSITION_T_BYTE_SIZE;
        let num_bits = position_t::from_be_bytes(num_bits_byte);

        let mut bits_len_bytes: [u8; USIZE_BYTE_SIZE] = Default::default();
        bits_len_bytes.copy_from_slice(&bytes[cursor..cursor+USIZE_BYTE_SIZE]);
        cursor += USIZE_BYTE_SIZE;
        let bits_len = usize::from_be_bytes(bits_len_bytes);

        let mut bits: Vec<word_t> = Vec::with_capacity(bits_len);
        for _ in 0..bits_len {
            let mut bits_word_bytes: [u8; WORD_T_BYTE_SIZE] = Default::default();
            bits_word_bytes.copy_from_slice(&bytes[cursor..cursor+WORD_T_BYTE_SIZE]);
            cursor += WORD_T_BYTE_SIZE;
            let word = word_t::from_be_bytes(bits_word_bytes);
            bits.push(word);
        }

        BitVector { num_bits, bits }
    }

    pub fn byte_size(&self) -> usize {
        let mut mem_size: usize = 0;
        #[allow(unused_unsafe)]
        unsafe {
            mem_size += size_of::<position_t>() + size_of_val(&*self.bits);
        }
        mem_size
    }

    pub fn new(
        bitvector_per_level: &Vec<Vec<word_t>>,
        num_bits_per_level: &Vec<position_t>,
        start_level: level_t,
        mut end_level: level_t,
    ) -> Self {
        if end_level == 0 {
            end_level = bitvector_per_level.len();
        }
        let num_bits = BitVector::total_num_bits(num_bits_per_level, start_level, end_level);
        let num_words = BitVector::num_words_from_num_bits(num_bits);
        let mut bit_vector = BitVector {
            num_bits: num_bits,
            bits: vec![0; num_words],
        };

        bit_vector.concatenate_bitvectors(
            bitvector_per_level,
            num_bits_per_level,
            start_level,
            end_level,
        );
        bit_vector
    }

    pub fn get_bits(&self) -> &Vec<word_t> {
        &self.bits
    }

    pub fn get_num_bits(&self) -> position_t {
        self.num_bits
    }

    fn total_num_bits(
        num_bits_per_level: &Vec<position_t>,
        start_level: level_t,
        end_level: level_t,
    ) -> position_t {
        let mut num_bits = 0;
        for level in start_level..end_level {
            num_bits += num_bits_per_level[level];
        }
        num_bits
    }

    fn num_words_from_num_bits(num_bits: position_t) -> position_t {
        if num_bits % K_WORD_SIZE == 0 {
            return num_bits / K_WORD_SIZE;
        } else {
            return num_bits / K_WORD_SIZE + 1;
        }
    }

    fn num_words(&self) -> position_t {
        if self.num_bits % K_WORD_SIZE == 0 {
            return self.num_bits / K_WORD_SIZE;
        } else {
            return self.num_bits / K_WORD_SIZE + 1;
        }
    }

    fn concatenate_bitvectors(
        &mut self,
        bitvector_per_level: &Vec<Vec<word_t>>,
        num_bits_per_level: &Vec<position_t>,
        start_level: level_t,
        end_level: level_t,
    ) {
        let mut bit_shift: position_t = 0;
        let mut word_id: position_t = 0;

        for level in start_level..end_level {
            if num_bits_per_level[level] == 0 {
                continue;
            }
            let num_complete_words: position_t = num_bits_per_level[level] / K_WORD_SIZE;
            for word in 0..num_complete_words {
                self.bits[word_id] |= bitvector_per_level[level][word] >> bit_shift;
                word_id += 1;
                if bit_shift > 0 {
                    self.bits[word_id] |=
                        bitvector_per_level[level][word] << (K_WORD_SIZE - bit_shift)
                }
            }

            let bits_remain: word_t = (num_bits_per_level[level] - num_complete_words * K_WORD_SIZE) as u64;
            if bits_remain > 0 {
                let last_word: word_t = bitvector_per_level[level][num_complete_words];
                self.bits[word_id] |= last_word >> bit_shift;
                if bit_shift as u64 + bits_remain < K_WORD_SIZE as u64 {
                    bit_shift += bits_remain as usize;
                } else {
                    word_id += 1;
                    self.bits[word_id] |= last_word << (K_WORD_SIZE - bit_shift);
                    bit_shift = bit_shift + bits_remain as usize - K_WORD_SIZE;
                }
            }
        }
    }

    pub fn read_bit(&self, pos: position_t) -> bool {
        let word_id: position_t = pos / K_WORD_SIZE;
        let offset: position_t = pos & (K_WORD_SIZE - 1);
        (self.bits[word_id] & (K_MSB_MASK >> offset)) != 0
    }

    pub fn distance_to_next_set_bit(&self, pos: position_t) -> position_t {
        let mut distance: position_t = 1;
        let mut word_id = (pos + 1) / K_WORD_SIZE;
        let offset = (pos + 1) % K_WORD_SIZE;

        let mut test_bits: word_t = self.bits[word_id] << offset;
        if test_bits > 0 {
            return distance + core::intrinsics::ctlz(test_bits) as usize;
        } else {
            if word_id == self.num_words() - 1 {
                return self.num_bits - pos
            }
            distance += K_WORD_SIZE - offset;
        }
        
        while word_id < self.num_words() - 1 {
            word_id += 1;
            test_bits = self.bits[word_id];
            if test_bits > 0 {
                return distance + core::intrinsics::ctlz(test_bits) as usize;
            }
            distance += K_WORD_SIZE;
        }
        return distance
    }
}