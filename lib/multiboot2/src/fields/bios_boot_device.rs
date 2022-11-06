use super::Tag;

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct BIOSBootDevice {
    tag: Tag,
    biosdev: u32,
    partition: u32,
    sub_partition: u32,
}
