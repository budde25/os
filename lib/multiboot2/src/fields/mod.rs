use core::fmt::Debug;
use core::marker::PhantomData;

mod apm;
mod basic_memory_info;
mod bios_boot_device;
mod boot_command_line;
mod boot_loader_name;
mod efi;
mod elf_symbols;
mod framebuffer_info;
mod image_load_base;
mod memory_map;
mod modules;
mod networking;
mod rsdp;
mod smbios;
mod vbe_info;

pub use {
    apm::APMTable,
    basic_memory_info::BasicMemoryInfo,
    bios_boot_device::BIOSBootDevice,
    boot_command_line::BootCommandLine,
    boot_loader_name::BootLoaderName,
    efi::{EFI32Image, EFI32Table, EFI64Image, EFI64Table, EFIError, EFIMemoryMap},
    elf_symbols::ElfSymbols,
    framebuffer_info::{FrameBufferInfo, FrameBufferType},
    image_load_base::ImageLoaderBase,
    memory_map::{MemoryMap, MemoryMapEntry, MemoryMapEntryType},
    modules::{Module, ModuleIter},
    networking::NetworkInfo,
    rsdp::{RsdpV1Tag, RsdpV2Tag},
    smbios::SMBIOSTables,
    vbe_info::VBEInfo,
};

/// A tag type, which represents the following structure and its size
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C, packed)]
pub struct Tag {
    r#type: TagType,
    size: u32,
}

impl Tag {
    pub fn terminator() -> Self {
        Self {
            r#type: TagType::Terminate,
            size: 8,
        }
    }

    pub fn tag_type(&self) -> TagType {
        self.r#type
    }

    pub fn size(&self) -> u32 {
        self.size
    }
}

#[derive(Clone)]
pub struct TagIter<'a> {
    current: *const Tag,
    phantom: PhantomData<&'a Tag>,
}

impl<'a> TagIter<'a> {
    pub(crate) fn new(start_tag: *const Tag) -> Self {
        Self {
            current: start_tag,
            phantom: PhantomData,
        }
    }
}

impl<'a> Iterator for TagIter<'a> {
    type Item = &'a Tag;

    fn next(&mut self) -> Option<Self::Item> {
        let cur = unsafe { &*self.current };
        match cur {
            &Tag {
                r#type: TagType::Terminate,
                size: 8,
            } => None,
            tag => {
                let mut addr = self.current as usize;
                addr += tag.size as usize;
                // 8 align
                addr = (addr + 7) & !7;

                self.current = addr as *const _;
                Some(tag)
            }
        }
    }
}

impl<'a> Debug for TagIter<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut list = f.debug_list();
        self.clone().for_each(|tag| {
            list.entry(&tag);
        });
        list.finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum TagType {
    Terminate = 0,
    BootCommandLine = 1,
    BootLoaderName = 2,
    Module = 3,
    BasicMemoryInfo = 4,
    BIOSBootDevice = 5,
    MemoryMap = 6,
    VBEInfo = 7,
    FrameBufferInfo = 8,
    ELFSymbols = 9,
    APMTable = 10,
    EFI32Table = 11,
    EFI64Table = 12,
    SMBIOSTables = 13,
    RSDPV1 = 14,
    RSDPV2 = 15,
    NetworkInfo = 16,
    EFIMemoryMap = 17,
    EFIError = 18,   // This tag indicates ExitBootServices wasn’t called
    EFI32Image = 19, // 32-bit image handle pointer
    EFI64Image = 20, // 64-bit image handle pointer
    ImageLoaderBase = 21,
}
