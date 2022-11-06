use super::Tag;

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct APMTable {
    tag: Tag,
    version: u16,
    cseg: u16,
    offset: u32,
    cseg_16: u16,
    dseg: u16,
    flags: u16,
    cseg_len: u16,
    cseg_16_len: u16,
    dseg_len: u16,
}
