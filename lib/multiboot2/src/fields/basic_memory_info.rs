use super::Tag;

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct BasicMemoryInfo {
    tag: Tag,
    upper: u32,
    lower: u32,
}
