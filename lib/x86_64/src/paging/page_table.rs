use crate::paging::phys_frame::PhysFrame;
use crate::PhysicalAddress;
use bitflags::bitflags;
use core::fmt::{self, Debug};
use core::marker::PhantomData;
use core::ops::{Index, IndexMut};

const PAGE_TABLE_ENTRY_COUNT: usize = 512;

pub struct PageTableEntry(u64);

impl PageTableEntry {
    pub fn new() -> Self {
        Self(0)
    }

    pub fn frame(&self) -> Option<PhysFrame> {
        if self.is_present() {
            let addr = PhysicalAddress::new(self.0 & 0x000f_ffff_ffff_f000);
            Some(PhysFrame::containing_address(addr))
        } else {
            None
        }
    }

    pub fn address(&self) -> PhysicalAddress {
        PhysicalAddress::new(self.0 & 0x000f_ffff_ffff_f000)
    }

    pub fn set_address(&mut self, addr: PhysicalAddress, flags: PageFlags) {
        assert!(addr.is_aligned(4096u64));
        self.0 = (u64::from(addr)) | flags.bits();
    }

    pub fn set_unused(&mut self) {
        self.0 = 0;
    }

    pub fn is_unused(&self) -> bool {
        self.0 == 0
    }

    pub fn flags(&self) -> PageFlags {
        PageFlags::from_bits_truncate(self.0)
    }

    pub fn set_flags(&mut self, flags: PageFlags) {
        self.0 |= flags.bits();
    }

    pub fn is_present(&self) -> bool {
        let flags = self.flags();
        flags.contains(PageFlags::PRESENT)
    }

    pub fn is_writable(&self) -> bool {
        let flags = self.flags();
        flags.contains(PageFlags::WRITEABLE)
    }

    pub fn is_user(&self) -> bool {
        let flags = self.flags();
        flags.contains(PageFlags::USER_ACCESSIBLE)
    }

    pub fn is_write_though_cache(&self) -> bool {
        let flags = self.flags();
        flags.contains(PageFlags::WRITE_THROUGH_CACHING)
    }

    pub fn is_chache_enabled(&self) -> bool {
        let flags = self.flags();
        !flags.contains(PageFlags::DISABLE_CACHE)
    }

    pub fn is_accessed(&self) -> bool {
        let flags = self.flags();
        flags.contains(PageFlags::ACCESSED)
    }

    pub fn is_dirty(&self) -> bool {
        let flags = self.flags();
        flags.contains(PageFlags::DIRTY)
    }

    pub fn is_huge(&self) -> bool {
        let flags = self.flags();
        flags.contains(PageFlags::HUGE_PAGE)
    }

    pub fn is_executable(&self) -> bool {
        let flags = self.flags();
        !flags.contains(PageFlags::NO_EXECUTE)
    }
}

impl Default for PageTableEntry {
    fn default() -> Self {
        Self::new()
    }
}

impl Debug for PageTableEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let flags = self.flags();
        let addr = self.frame();
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

pub trait HierarchicalLevel: TableLevel {
    type NextLevel: TableLevel;
}

impl HierarchicalLevel for Level4 {
    type NextLevel = Level3;
}

impl HierarchicalLevel for Level3 {
    type NextLevel = Level2;
}

impl HierarchicalLevel for Level2 {
    type NextLevel = Level1;
}

pub trait TableLevel {}

pub enum Level4 {}
pub enum Level3 {}
pub enum Level2 {}
pub enum Level1 {}

impl TableLevel for Level4 {}
impl TableLevel for Level3 {}
impl TableLevel for Level2 {}
impl TableLevel for Level1 {}

#[derive(Debug)]
#[repr(C, align(4096))]
pub struct PageTable<L: TableLevel> {
    entries: [PageTableEntry; PAGE_TABLE_ENTRY_COUNT],
    _level: PhantomData<L>,
}

impl<L> PageTable<L>
where
    L: TableLevel,
{
    pub fn zero(&mut self) {
        for entry in self.entries.iter_mut() {
            entry.set_unused()
        }
    }

    /// Returns an iterator over the entries of the page table.
    pub fn iter(&self) -> impl Iterator<Item = &PageTableEntry> {
        self.entries.iter()
    }

    /// Returns an iterator that allows modifying the entries of the page table.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut PageTableEntry> {
        self.entries.iter_mut()
    }
}

impl<L> PageTable<L>
where
    L: HierarchicalLevel,
{
    pub fn next_table(&self, index: usize) -> Option<&PageTable<L::NextLevel>> {
        self.next_table_address(index)
            .map(|addr| unsafe { &*(u64::from(addr) as *const _) })
    }

    pub fn next_table_mut(&self, index: usize) -> Option<&mut PageTable<L::NextLevel>> {
        self.next_table_address(index)
            .map(|addr| unsafe { &mut *(u64::from(addr) as *mut _) })
    }

    pub fn next_table_address(&self, index: usize) -> Option<PhysicalAddress> {
        let entry = &self[index];
        if entry.is_huge() || !entry.is_present() {
            return None;
        }

        Some(entry.address())
    }
}

impl<L> Index<usize> for PageTable<L>
where
    L: TableLevel,
{
    type Output = PageTableEntry;
    fn index(&self, index: usize) -> &Self::Output {
        &self.entries[index]
    }
}

impl<L> IndexMut<usize> for PageTable<L>
where
    L: TableLevel,
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.entries[index]
    }
}

impl<L> Index<PageTableIndex> for PageTable<L>
where
    L: TableLevel,
{
    type Output = PageTableEntry;
    fn index(&self, index: PageTableIndex) -> &Self::Output {
        &self.entries[usize::from(index)]
    }
}

impl<L> IndexMut<PageTableIndex> for PageTable<L>
where
    L: TableLevel,
{
    fn index_mut(&mut self, index: PageTableIndex) -> &mut Self::Output {
        &mut self.entries[usize::from(index)]
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
        Self(index % 4096)
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
