use alloc::vec::Vec;
use crate::OFFSET_MASK;

pub struct Bitmap64 {
    map: Vec<u64>,
}

impl Bitmap64 {
    pub const BITS: u32 = u64::BITS;

    fn word_filled_by(bit: bool) -> u64 {
        if bit { !0u64 } else { 0u64 }
    }

    fn required_word_len(bit_len: u32) -> u32 {
        (bit_len + Bitmap64::BITS - 1) / Bitmap64::BITS
    }

    pub fn new(word_len: u32, value: bool) -> Bitmap64 {
        Self {
            map: vec![Bitmap64::word_filled_by(value), word_len as u64],
        }
    }

    pub fn resize_words(&mut self, new_words: u32, value: bool) {
        self.map.resize(new_words as usize, Bitmap64::word_filled_by(value));
    }

    #[inline(always)]
    pub fn get_word(&self, b: u32) -> u64 {
        if b as usize >= self.map.len() {
            0u64
        } else {
            self.map[b as usize]
        }
    }

    #[inline(always)]
    pub fn get_bit(&self, i: u32) -> bool {
        let q = i / Bitmap64::BITS;
        let r = i % Bitmap64::BITS;
        let w = self.get_word(q);
        let mask = 1u64 << r;
        (w & mask) != 0
    }

    #[inline(always)]
    pub fn set_bit(&mut self, i: u32, b: bool) {
        let q = i / Bitmap64::BITS;
        let r = i % Bitmap64::BITS;
        let w = self.get_word(q);
        let mask = !(1u64 << r);
        let bit = u64::from(b) << r;
        self.map[q as usize] = (w & mask) | bit;
    }
}

pub struct BPXChecker {
    bitmap: Bitmap64,
}

impl BPXChecker {
    pub fn new(len: u32) -> BPXChecker {
        Self {
            bitmap: Bitmap64::new(Bitmap64::required_word_len(len), false),
        }
    }

    pub fn resize(&mut self, new_len: u32) {
        debug_assert_eq!(new_len % Bitmap64::BITS, 0);
        self.bitmap.resize_words(Bitmap64::required_word_len(new_len), false);
    }

    pub fn set_bit(&mut self, i: u32, value: bool) {
        self.bitmap.set_bit(i, value);
    }

    const MASKS: [u64; 6] = [
        0b0101u64 * 0x1111111111111111u64,
        0b0011u64 * 0x1111111111111111u64,
        0x0F0F0F0F0F0F0F0Fu64,
        0x00FF00FF00FF00FFu64,
        0x0000FFFF0000FFFFu64,
        0x00000000FFFFFFFFu64, // never used
    ];
    const NO_CANDIDATES: u64 = !0u64;

    pub fn find_base_collectively(self, base_origin: u32, labels: &[u32]) -> u64 {
        debug_assert_eq!(base_origin % Bitmap64::BITS, 0);
        let mut x = 0u64;
        for &label in labels {
            let q = (base_origin ^ label) / Bitmap64::BITS;
            let mut w: u64 = self.bitmap.get_word(q);
            // Block-wise swap
            for i in 0..5 {
                let width = 1u32 << i;
                if label & width != 0 {
                    w = ((w >> width) & BPXChecker::MASKS[i]) | ((w & BPXChecker::MASKS[i]) << width);
                }
            }
            if label & (1u32 << 5) != 0 {
                w = (w >> 32) | (w << 32);
            }
            // Merge invalid xor-maps
            x |= w;
            if x == BPXChecker::NO_CANDIDATES { break; }
        }
        x
    }

    pub fn verify_base_collectively(self, base_origin: u32, labels: &[u32]) -> Option<u32> {
        if base_origin & !OFFSET_MASK != 0 {
            return None;
        }
        for &label in labels {
            if label & !OFFSET_MASK != 0 {
                return None;
            }
        }
        let x = self.find_base_collectively(base_origin & !0x00111111u32, labels);
        if x != BPXChecker::NO_CANDIDATES {
            Some(base_origin ^ x.leading_ones())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitparallel_xcheck() {
        let map = [1,0,0,1,0,0,1,0,1,0,1,0,0,0,1,0,0,0,0,1,0,0,1,0,1,0,0,0,0,0,1,1,0,0,0,1,0,1,0,0,0,1,0,1,0,1,0,0,0,0,0,1,0,0,0,0,0,0,0,0,1,0,0,1];
        let labels = [1, 3, 7, 9, 11, 23, 41];
        let expected_bases = [6, 14, 37, 45, 51, 57];

        let mut xc = BPXChecker::new(1);
        for i in 0..64 { xc.set_bit(i, map[i as usize] != 0); }
        let x = xc.find_base_collectively(0, &labels);
        let mut candidates = vec![];
        for i in 0..64 {
            if (1u64 << i) & x == 0 {
                candidates.push(i);
            }
        }
        assert_eq!(expected_bases.len(), candidates.len());
        for i in 0..expected_bases.len() {
            assert_eq!(expected_bases[i], candidates[i]);
        }
    }
}
