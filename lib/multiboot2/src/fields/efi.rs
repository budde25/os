use super::Tag;

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct EFIMemoryMap {
    tag: Tag,
    descriptor_size: u32,
    descriptor_version: u32,
    entries: [u8; 0], // FIXME allow for data
}

#[derive(Debug, Clone, Copy)]
pub struct EFIError {
    tag: Tag,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct EFI32Image {
    tag: Tag,
    pointer: u32,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct EFI64Image {
    tag: Tag,
    pointer: u64,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct EFI32Table {
    tag: Tag,
    address_ptr: u32,
}

impl EFI32Table {
    pub fn address(&self) -> usize {
        usize::try_from(self.address_ptr).unwrap()
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct EFI64Table {
    tag: Tag,
    address_ptr: u64,
}

impl EFI64Table {
    pub fn address(&self) -> usize {
        usize::try_from(self.address_ptr).unwrap()
    }
}
