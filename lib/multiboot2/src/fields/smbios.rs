use super::Tag;

use core::fmt::Debug;

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct SMBIOSTables {
    tag: Tag,
    major: u8,
    minor: u8,
    _reserved: [u8; 6],
    smbios_tables: [u8; 0], // FIXME: allow for data
}

impl Debug for SMBIOSTables {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SMBIOSTables")
            .field("tag", &self.tag)
            .field("major", &self.major)
            .field("minor", &self.minor)
            .finish_non_exhaustive()
    }
}
