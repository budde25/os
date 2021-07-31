#[repr(C, packed)]
struct MultipleApicDescriptorTable {
    signature: [char; 4],
    length: u32,
    revision: u8,
    checksum: u8,
    oem_id: [char; 6],
    oem_table_id: [char; 8],
    oem_revision: u32,
    creator_id: u32,
    creator_revision: u32,
    lapic_addr: u32,
    flags: u32,
}

struct EntryHeader {
    entry_type: u8,
    length: u8,
}
