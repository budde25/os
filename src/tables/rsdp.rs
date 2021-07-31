#[repr(C, packed)]
struct RSDPDescriptor {
    signature: [char; 8],
    checksum: u8,
    oem_id: [char; 6],
    revision: u8,
    rsdt_address: u32,
    // only if revision 2
    length: u32,
    xsdt_address: u64,
    extended_checksum: u8,
    _reserved: [u8; 3],
}
