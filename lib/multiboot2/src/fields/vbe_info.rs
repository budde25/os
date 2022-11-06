use super::Tag;

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct VBEInfo {
    tag: Tag,
    mode: u16,
    interface_seg: u16,
    interface_offset: u16,
    interface_len: u16,
    control_info: [u8; 512],
    mode_info: [u8; 256],
}
