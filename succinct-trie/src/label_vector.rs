use crate::config::*;

pub struct LabelVector {
    labels: Vec<label_t>,
}

impl LabelVector {
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
        let mut updated_pos = pos;
        let mut updated_search_len = search_len;
        if updated_search_len > 1 && self.labels[updated_pos] == K_TERMINATOR {
            updated_pos += 1;
            updated_search_len -= 1;
        }

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

    pub fn read(&self, pos: position_t) -> label_t {
        self.labels[pos]
    }
}
