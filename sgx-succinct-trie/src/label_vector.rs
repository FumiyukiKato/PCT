use core::intrinsics::{size_of_val};
use std::vec::Vec;
use crate::config::*;

pub struct LabelVector {
    labels: Vec<label_t>,
}

impl LabelVector {
    pub fn serialize(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::with_capacity(1000);
        bytes.extend(self.labels.len().to_be_bytes().iter());
        for bit in self.labels.iter() {
            bytes.extend(bit.to_be_bytes().iter());
        }
        bytes
    }

    pub fn deserialize(bytes: &[u8]) -> Self {
        let mut cursor = 0;

        let mut labels_len_bytes: [u8; USIZE_BYTE_SIZE] = Default::default();
        labels_len_bytes.copy_from_slice(&bytes[cursor..cursor+USIZE_BYTE_SIZE]);
        cursor += USIZE_BYTE_SIZE;
        let labels_len = usize::from_be_bytes(labels_len_bytes);

        let mut labels: Vec<label_t> = Vec::with_capacity(labels_len);
        for _ in 0..labels_len {
            let mut labels_bytes: [u8; LABEL_T_BYTE_SIZE] = Default::default();
            labels_bytes.copy_from_slice(&bytes[cursor..cursor+LABEL_T_BYTE_SIZE]);
            cursor += LABEL_T_BYTE_SIZE;
            let label = label_t::from_be_bytes(labels_bytes);
            labels.push(label);
        }

        LabelVector { labels }
    }

    pub fn byte_size(&self) -> usize {
        let mut mem_size = 0;
        #[allow(unused_unsafe)]
        unsafe {
            mem_size += size_of_val(&*self.labels);
        }
        mem_size
    }

    pub fn new(
        labels_per_level: &Vec<Vec<label_t>>,
        start_level: level_t,
        mut end_level: level_t,
    ) -> LabelVector {
        if end_level == 0 {
            end_level = labels_per_level.len()
        };
        let mut num_bytes = 1;
        for level in start_level..end_level {
            num_bytes += labels_per_level[level].len();
        }

        // let mut labels = vec![0; num_bytes];
        let mut labels = vec![0; num_bytes + 16]; // Modified by Shunsuke Kanda (+16 is for avoiding heap overflow in SIMD)

        let mut pos: position_t = 0;
        for level in start_level..end_level {
            for idx in 0..labels_per_level[level].len() {
                labels[pos] = labels_per_level[level][idx];
                pos += 1;
            }
        }

        LabelVector { labels }
    }

    pub fn search(&self, target: label_t, pos: position_t, search_len: position_t) -> (bool, position_t) {
        let updated_pos = pos;
        let updated_search_len = search_len;


        if updated_search_len < 3 {
            return self.linear_search(target, updated_pos, updated_search_len);
        } else if updated_search_len < 12 {
            return self.binary_search(target, updated_pos, updated_search_len);
        } else {
            return self.simd_search(target, updated_pos, updated_search_len);
        }
    }

    fn linear_search(&self, target: label_t, mut pos: position_t, search_len: position_t) -> (bool, position_t) {
        for i in 0..search_len {
            if target == self.labels[pos + i] {
                pos += i;
                return (true, pos)
            }
        }
        return (false, pos)
    }

    fn binary_search(&self, target: label_t, mut pos: position_t, search_len: position_t) -> (bool, position_t) {
        let mut l = pos;
        let mut r = pos + search_len;

        while l < r {
            let m = (l + r) >> 1;
            if target < self.labels[m] {
                r = m;
            } else if target == self.labels[m] {
                pos = m;
                return (true, pos)
            } else {
                l = m + 1;
            }
        }
        return (false, pos)
    }

    fn simd_search(&self, target: label_t, mut pos: position_t, search_len: position_t) -> (bool, position_t) {
        let mut num_labels_searched = 0;
        let mut num_labels_left = search_len;

        unsafe {
            while (num_labels_left >> 4) > 0 {
                let start_ptr = self
                    .labels
                    .as_ptr()
                    .offset((pos + num_labels_searched) as isize);
                let cmp = core::arch::x86_64::_mm_cmpeq_epi8(
                    core::arch::x86_64::_mm_set1_epi8(target as i8),
                    core::arch::x86_64::_mm_loadu_si128(
                        start_ptr as *const core::arch::x86_64::__m128i,
                    ),
                );
                let check_bits: i32 = core::arch::x86_64::_mm_movemask_epi8(cmp);
                if check_bits != 0 {
                    pos += num_labels_searched +  core::intrinsics::cttz(check_bits) as usize;
                    return (true, pos)
                }

                num_labels_searched += 16;
                num_labels_left -= 16;
            }

            if num_labels_left > 0 {
                let start_ptr = self.labels.as_ptr().offset((pos + num_labels_searched) as isize);
                let cmp = core::arch::x86_64::_mm_cmpeq_epi8(
                    core::arch::x86_64::_mm_set1_epi8(target as i8),
                    core::arch::x86_64::_mm_loadu_si128(
                        start_ptr as *const core::arch::x86_64::__m128i,
                    ),
                );
                let leftover_bits_mask = (1 << num_labels_left) - 1;
                let check_bits = core::arch::x86_64::_mm_movemask_epi8(cmp) & leftover_bits_mask;
                if check_bits != 0 {
                    pos += num_labels_searched + core::intrinsics::cttz(check_bits) as usize;
                    return (true, pos)
                }
            }
        }
        return (false, pos)
    }
}
