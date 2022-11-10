use core::mem::size_of;

const MULTIBOOT2_MAGIC: u32 = 0xE85250D6;

#[repr(C, align(8))]
#[derive(Debug, Clone)]
pub struct MultiBoot2Header {
    /// Multiboot2 Header magic
    magic: u32,
    /// Defines the arch
    architecture: u32,
    /// the size of the struct
    header_length: u32,
    /// checksum + arch + magic + header_length should equal 0
    checksum: u32,
    /// optinal tags, (we only want the end tag)
    tag: [HeaderTag; 1],
}

impl MultiBoot2Header {
    pub const fn new() -> Self {
        let arch = 0; // 32-bit (protected mode) of i386
        let magic = MULTIBOOT2_MAGIC;
        let header_length = size_of::<Self>() as u32;

        let checksum: u32 = (u32::MAX - (magic + arch + header_length)) + 1;

        Self {
            magic,
            architecture: arch,
            header_length,
            checksum,
            tag: [HeaderTag::end_tag(); 1],
        }
    }
}

impl Default for MultiBoot2Header {
    fn default() -> Self {
        Self::new()
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct HeaderTag {
    r#type: u16,
    flags: u16,
    size: u32,
}

impl HeaderTag {
    const fn end_tag() -> Self {
        Self {
            r#type: 0,
            flags: 0,
            size: 8,
        }
    }
}

impl Default for HeaderTag {
    fn default() -> Self {
        Self::end_tag()
    }
}