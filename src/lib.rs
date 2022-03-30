pub mod builder;
mod mapper;
pub mod mpftrie;
pub mod mptrie;
pub mod trie;
mod utils;

pub const OFFSET_MASK: u32 = 0x7fff_ffff;
pub const INVALID_IDX: u32 = 0xffff_ffff;
pub const END_MARKER: u32 = 0;
pub const END_CODE: u32 = 0;

pub use mpftrie::MpfTrie;
pub use mptrie::MpTrie;
pub use trie::Trie;

#[derive(Default, Clone, Copy)]
pub struct Node {
    pub(crate) base: u32,
    pub(crate) check: u32,
}

impl Node {
    #[inline(always)]
    pub const fn get_base(&self) -> u32 {
        self.base & OFFSET_MASK
    }

    #[inline(always)]
    pub const fn get_check(&self) -> u32 {
        self.check & OFFSET_MASK
    }

    #[inline(always)]
    pub const fn is_leaf(&self) -> bool {
        self.base & !OFFSET_MASK != 0
    }

    #[inline(always)]
    pub const fn has_leaf(&self) -> bool {
        self.check & !OFFSET_MASK != 0
    }

    #[inline(always)]
    pub const fn is_vacant(&self) -> bool {
        self.base == OFFSET_MASK && self.check == OFFSET_MASK
    }
}
