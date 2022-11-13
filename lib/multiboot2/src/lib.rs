#![no_std]

use core::fmt::Debug;

pub use fields::{
    APMTable, BIOSBootDevice, BasicMemoryInfo, BootCommandLine, BootLoaderName, EFI32Image,
    EFI32Table, EFI64Image, EFI64Table, EFIError, EFIMemoryMap, ElfSymbols, FrameBufferInfo,
    FrameBufferType, ImageLoaderBase, MemoryMap, MemoryMapEntry, MemoryMapEntryType, Module,
    NetworkInfo, RsdpV1Tag, RsdpV2Tag, SMBIOSTables, VBEInfo,
};
use fields::{ModuleIter, Tag, TagIter, TagType};
pub use header::MultiBoot2Header;

mod fields;
mod header;

#[derive(Clone, Copy)]
pub struct MultibootInfo {
    inner: *const MultibootTable,
}

impl MultibootInfo {
    /// # Safety
    /// * Must be 8 byte aligned as defined by the spec
    /// * Must point to a valid multiboot table
    /// * Must be readable memory
    /// * Memory must not change after loading, aka read only
    pub unsafe fn new(address: usize) -> Result<Self, MultibootInfoLoadError> {
        if address % 8 != 0 || address == 0 {
            return Err(MultibootInfoLoadError::InavalidAddress);
        }

        let mbinfo = Self {
            inner: address as *const _,
        };

        let total_size = mbinfo.get().total_size;
        if total_size % 8 != 0 {
            return Err(MultibootInfoLoadError::InavalidSize(total_size));
        }

        let end_tag = unsafe { &*((mbinfo.end_address() - 8) as *const Tag) };
        if end_tag != &Tag::terminator() {
            return Err(MultibootInfoLoadError::MissingTerminator);
        }

        Ok(mbinfo)
    }

    /// Get the start address of the MultibootTable
    pub fn start_address(&self) -> usize {
        self.inner as usize
    }

    /// Get the end address of the MultibootTable
    pub fn end_address(&self) -> usize {
        self.start_address() + self.get().total_size() as usize
    }

    /// Get the total size of the structure
    pub fn total_size(&self) -> u32 {
        self.get().total_size()
    }

    /// Search for the BootLoader
    pub fn boot_loader_name(&self) -> Option<&BootLoaderName> {
        self.get_tag(TagType::BootLoaderName)
            .map(|tag| unsafe { &*(tag as *const Tag as *const BootLoaderName) })
    }

    /// Search for the BIOSBootDevice
    pub fn bios_boot_device(&self) -> Option<&BIOSBootDevice> {
        self.get_tag(TagType::BIOSBootDevice)
            .map(|tag| unsafe { &*(tag as *const Tag as *const BIOSBootDevice) })
    }

    /// Search for the BootCommandLine
    pub fn boot_loader_command_line(&self) -> Option<&BootCommandLine> {
        self.get_tag(TagType::BootCommandLine)
            .map(|tag| unsafe { &*(tag as *const Tag as *const BootCommandLine) })
    }

    /// Get an iterator for all of the modules
    pub fn modules(&self) -> ModuleIter {
        ModuleIter::new(TagIter::new(unsafe { self.inner.offset(1) } as *const _))
    }

    /// Search for RsdpV1
    pub fn rsdp_v1(&self) -> Option<&RsdpV1Tag> {
        self.get_tag(TagType::RSDPV1)
            .map(|tag| unsafe { &*(tag as *const Tag as *const RsdpV1Tag) })
    }

    /// Search for RsdpV2
    pub fn rsdp_v2(&self) -> Option<&RsdpV2Tag> {
        self.get_tag(TagType::RSDPV2)
            .map(|tag| unsafe { &*(tag as *const Tag as *const RsdpV2Tag) })
    }

    fn get(&self) -> &MultibootTable {
        unsafe { &*self.inner }
    }

    /// Get the first occurrence of a given tag
    fn get_tag(&self, tag_type: TagType) -> Option<&Tag> {
        self.tags().find(|tag| tag.tag_type() == tag_type)
    }

    /// Get an iterator of all the tags
    fn tags(&self) -> TagIter {
        TagIter::new(unsafe { self.inner.offset(1) } as *const _)
    }
}
/// This cannot be mutated so it is safe to send between threads
unsafe impl Send for MultibootInfo {}
unsafe impl Sync for MultibootInfo {}

// The header of the multiboot table
#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct MultibootTable {
    total_size: u32,
    _reserved: u32,
}

impl MultibootTable {
    pub fn total_size(&self) -> u32 {
        self.total_size
    }
}

impl Debug for MultibootTable {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let size = self.total_size;
        f.debug_struct("MultibootTable")
            .field("total_size", &size)
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MultibootInfoLoadError {
    /// Address must not be null or None 8-byte aligned
    InavalidAddress,
    /// The size of the structure is invalid (must be a multiple of 8)
    InavalidSize(u32),
    /// Multiboot structure is missing the Termination Tag
    MissingTerminator,
}
