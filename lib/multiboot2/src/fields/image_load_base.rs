use super::Tag;

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct ImageLoaderBase {
    tag: Tag,
    load_base_addr: u32,
}
