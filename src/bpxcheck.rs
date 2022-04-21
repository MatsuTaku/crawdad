use alloc::vec::Vec;
use crate::OFFSET_MASK;

#[derive(Default)]
pub struct BPXChecker {
    pub bitmap: Vec<u64>,
}

impl BPXChecker {
    pub const BITS: u32 = u64::BITS;

    #[inline(always)]
    const fn word_filled_by(bit: bool) -> u64 {
        if bit { !0u64 } else { 0u64 }
    }

    fn required_word_len(len: usize) -> usize {
        (len + BPXChecker::BITS as usize - 1) / BPXChecker::BITS as usize
    }

    #[inline(always)]
    pub fn word_index(i: u32) -> u32 {
        i / BPXChecker::BITS
    }

    #[inline(always)]
    fn word_offset(i: u32) -> u32 {
        i % BPXChecker::BITS
    }

    #[inline(always)]
    fn index_pair(i: u32) -> (u32, u32) {
        (BPXChecker::word_index(i), BPXChecker::word_offset(i))
    }

    pub fn new(len: usize) -> Self {
        Self {
            bitmap: vec![BPXChecker::word_filled_by(false); BPXChecker::required_word_len(len)],
        }
    }

    #[inline(always)]
    pub fn get_word(&self, wi: u32) -> u64 {
        if wi & !OFFSET_MASK != 0 {
            BPXChecker::word_filled_by(true)
        } else if wi as usize >= self.bitmap.len() {
            BPXChecker::word_filled_by(false)
        } else {
            self.bitmap[wi as usize]
        }
    }

    pub fn is_fixed(&self, i: u32) -> bool {
        let (q, r) = BPXChecker::index_pair(i);
        self.get_word(q) & (1u64 << r) != 0
    }

    #[inline(always)]
    pub fn set_fixed(&mut self, i: u32) {
        let (q, r) = BPXChecker::index_pair(i);
        self.bitmap[q as usize] |= 1u64 << r;
    }

    pub fn resize(&mut self, new_len: usize) {
        self.bitmap.resize(BPXChecker::required_word_len(new_len), BPXChecker::word_filled_by(false));
    }

    const BLOCK_MASKS: [u64; 6] = [
        0b0101u64 * 0x1111111111111111u64,
        0b0011u64 * 0x1111111111111111u64,
        0x0F0F0F0F0F0F0F0Fu64,
        0x00FF00FF00FF00FFu64,
        0x0000FFFF0000FFFFu64,
        0x00000000FFFFFFFFu64, // never used
    ];
    pub const NO_CANDIDATE: u64 = BPXChecker::word_filled_by(true);

    #[inline(always)]
    pub fn disabled_base_mask(&self, base_origin: u32, labels: &[u32]) -> u64 {
        debug_assert_eq!(base_origin % BPXChecker::BITS, 0);

        let mut x = 0u64;
        for &label in labels {
            let q = BPXChecker::word_index(base_origin ^ label);
            let mut w: u64 = self.get_word(q);
            // Block-wise swap
            for i in 0..5 {
                let width = 1u32 << i;
                if label & width != 0 {
                    w = ((w >> width) & BPXChecker::BLOCK_MASKS[i]) | ((w & BPXChecker::BLOCK_MASKS[i]) << width);
                }
            }
            if label & (1u32 << 5) != 0 {
                w = (w >> 32) | (w << 32);
            }
            // Merge invalid xor-maps
            x |= w;
            if x == BPXChecker::NO_CANDIDATE { break; }
        }
        x
    }

    pub const BASE_MASK: u32 = !(BPXChecker::BITS - 1);

    #[inline(always)]
    pub fn find_base_for_64adjacent(&self, base_origin: u32, labels: &[u32]) -> Option<u32> {
        let base_front = base_origin & BPXChecker::BASE_MASK;
        let x = self.disabled_base_mask(base_front, labels);
        if x != BPXChecker::NO_CANDIDATE {
            Some(base_front ^ x.trailing_ones()) // Return one of the candidate
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_base_64adjacent() {
        let map = [1,0,0,1,0,0,1,0,1,0,1,0,0,0,1,0,0,0,0,1,0,0,1,0,1,0,0,0,0,0,1,1,0,0,0,1,0,1,0,0,0,1,0,1,0,1,0,0,0,0,0,1,0,0,0,0,0,0,0,0,1,0,0,1];
        let labels = [1, 3, 7, 9, 11, 23, 41];
        let expected_bases = [6, 14, 37, 45, 51, 57];

        let mut xc = BPXChecker::new(64);
        for i in 0..64 {
            if map[i] != 0 {
                xc.set_fixed(i as u32);
            }
        }
        let x = xc.disabled_base_mask(0, &labels);
        let mut candidate = vec![];
        for i in 0..64 {
            if x & (1u64 << i) == 0 {
                candidate.push(i);
            }
        }
        assert_eq!(expected_bases.len(), candidate.len());
        for i in 0..expected_bases.len() {
            assert_eq!(expected_bases[i], candidate[i]);
        }
    }
}
