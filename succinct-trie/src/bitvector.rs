use crate::config::*;

pub struct BitVector {
    num_bits: position_t,
    bits: Vec<word_t>,
}

impl BitVector{
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
        
        while word_id < self.num_words() {
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
