use super::Tag;

/// Apears once per card
/// FIXME: allow multiple networks infos
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct NetworkInfo {
    tag: Tag,
    dhcp_ack: [u8; 0], // FIXME allow for data
}
