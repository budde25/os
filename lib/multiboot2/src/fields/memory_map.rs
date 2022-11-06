use super::Tag;

use core::fmt::Debug;

#[derive()]
#[repr(C, packed)]
pub struct MemoryMap {
    tag: Tag,
    entry_size: u32,
    entry_version: u32,
    entries: [u8; 0], // slice of entries
}

impl MemoryMap {
    // TODO: make sure this is correct
    pub fn entries(&self) -> &[MemoryMapEntry] {
        let entry_count = (self.tag.size() - 16) / self.entry_size;
        unsafe {
            core::slice::from_raw_parts(
                &self.entries as *const _ as *const MemoryMapEntry,
                entry_count as usize,
            )
        }
    }
}

impl Debug for MemoryMap {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let entry_size = self.entry_size;
        let entry_version = self.entry_version;
        f.debug_struct("MemoryMap")
            .field("entry_size", &entry_size)
            .field("entry_version", &entry_version)
            .finish()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(C, packed)]
pub struct MemoryMapEntry {
    base_addr: u64,
    length: u64,
    r#type: u32,
    _reserved: u32,
}

impl MemoryMapEntry {
    fn address(&self) -> usize {
        usize::try_from(self.base_addr).unwrap()
    }

    fn length(&self) -> u64 {
        self.length
    }

    fn entry_type(&self) -> MemoryMapEntryType {
        self.r#type.into()
    }
}

impl Debug for MemoryMapEntry {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("MemoryMapEntry")
            .field("address", &self.address())
            .field("length", &self.length())
            .field("entry_type", &self.entry_type())
            .finish()
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub enum MemoryMapEntryType {
    BootloaderReserved = 0,
    AvailableRam = 1,
    AcpiInfo = 3,
    Reserved = 4,
    DefectiveRam = 5,
}

impl From<u32> for MemoryMapEntryType {
    fn from(i: u32) -> Self {
        match i {
            0 => Self::BootloaderReserved,
            1 => Self::AvailableRam,
            3 => Self::AcpiInfo,
            4 => Self::DefectiveRam,
            _ => Self::Reserved,
        }
    }
}
