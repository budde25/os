use crate::address::phys::PhysicalAddress;
use bitflags::bitflags;
use core::fmt::{self, Debug};
use core::ops::{Index, IndexMut};

const PAGE_TABLE_ENTRY_COUNT: usize = 512;

pub struct PageTableEntry(u64);

impl PageTableEntry {
    fn get_physical_addr(&self) -> PhysicalAddress {
        PhysicalAddress::new(self.0 & 0x000f_ffff_ffff_f000)
    }
}

impl Debug for PageTableEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let flags = PageFlags::from_bits_truncate(self.0);
        let addr = self.get_physical_addr();
        f.debug_struct("Page")
            .field("Address", &addr)
            .field("Flags", &flags)
            .finish()
    }
}

bitflags! {
    pub struct PageFlags: u64 {
        const PRESENT = 1;
        const WRITEABLE = 1 << 1;
        const USER_ACCESSIBLE = 1 << 2;
        const WRITE_THROUGH_CACHING = 1 << 3;
        const DISABLE_CACHE = 1 << 4;
        const ACCESSED = 1 << 5; // set by cpu
        const DIRTY = 1 << 6; //set by cpu
        const HUGE_PAGE = 1 << 7; // must be zero on pe, pdpte
        const GLOBAL = 1 << 8;
        const AVAILABLE_1 = 1 << 9;
        const AVAILABLE_2 = 1 << 10;
        const AVAILABLE_3 = 1 << 11;
        const AVAILABLE_4 = 1 << 52;
        const AVAILABLE_5 = 1 << 53;
        const AVAILABLE_6 = 1 << 54;
        const AVAILABLE_7 = 1 << 55;
        const AVAILABLE_8 = 1 << 56;
        const AVAILABLE_9 = 1 << 57;
        const AVAILABLE_10 = 1 << 58;
        const AVAILABLE_11 = 1 << 59;
        const AVAILABLE_12 = 1 << 60;
        const AVAILABLE_13 = 1 << 61;
        const AVAILABLE_14 = 1 << 62;
        const NO_EXECUTE = 1 << 63;
    }
}

#[derive(Debug)]
#[repr(align(4096))]
#[repr(C)]
pub struct PageTable {
    entries: [PageTableEntry; PAGE_TABLE_ENTRY_COUNT],
}

impl Index<usize> for PageTable {
    type Output = PageTableEntry;
    fn index(&self, index: usize) -> &Self::Output {
        &self.entries[index]
    }
}

impl IndexMut<usize> for PageTable {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.entries[index]
    }
}

/// Guaranteed to only ever contain 0..512
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PageTableIndex(u16);

impl PageTableIndex {
    pub fn new(index: u16) -> Self {
        assert!(usize::from(index) < PAGE_TABLE_ENTRY_COUNT);
        Self(index)
    }

    /// Creates a new index from the given `u16`. Throws away bits if the value is >=512.
    pub const fn new_truncate(index: u16) -> Self {
        Self(index % PAGE_TABLE_ENTRY_COUNT as u16)
    }
}

impl From<PageTableIndex> for u16 {
    fn from(index: PageTableIndex) -> Self {
        index.0
    }
}

impl From<PageTableIndex> for u32 {
    fn from(index: PageTableIndex) -> Self {
        u32::from(index.0)
    }
}

impl From<PageTableIndex> for u64 {
    fn from(index: PageTableIndex) -> Self {
        u64::from(index.0)
    }
}

impl From<PageTableIndex> for usize {
    fn from(index: PageTableIndex) -> Self {
        usize::from(index.0)
    }
}

//Guaranteed to only ever contain 0..4096
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PageOffset(u16);

impl PageOffset {
    pub fn new(index: u16) -> Self {
        assert!(usize::from(index) < 4096);
        Self(index)
    }

    /// Creates a new index from the given `u16`. Throws away bits if the value is >=4096.
    pub const fn new_truncate(index: u16) -> Self {
        Self(index % 4096 as u16)
    }
}

impl From<PageOffset> for u16 {
    fn from(index: PageOffset) -> Self {
        index.0
    }
}

impl From<PageOffset> for u32 {
    fn from(index: PageOffset) -> Self {
        u32::from(index.0)
    }
}

impl From<PageOffset> for u64 {
    fn from(index: PageOffset) -> Self {
        u64::from(index.0)
    }
}

impl From<PageOffset> for usize {
    fn from(index: PageOffset) -> Self {
        usize::from(index.0)
    }
}
