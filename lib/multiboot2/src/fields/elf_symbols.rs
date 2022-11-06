use super::Tag;

use core::fmt::Debug;

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct ElfSymbols {
    tag: Tag,
    num: u16,
    entsize: u16,
    shndx: u16,
    _reserved: u16,
    section_headers: [u8; 0], // TODO: learn how to parse elf symbols
}

impl Debug for ElfSymbols {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let num = self.num;
        let entsize = self.entsize;
        let shndx = self.shndx;
        f.debug_struct("ElfSymbols")
            .field("tag", &self.tag)
            .field("num", &num)
            .field("entsize", &entsize)
            .field("shndx", &shndx)
            .finish_non_exhaustive()
    }
}
