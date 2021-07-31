#[repr(C, packed)]
struct FloatingPointerStructure {
    signature: [char; 4], // "_MP_" or 0x5F504D5F.
    configuration_table: u32,
    length: u8, // In 16 bytes (e.g. 1 = 16 bytes, 2 = 32 bytes)
    mp_specification_revision: u8,
    checksum: u8, // This value should make all bytes in the table equal 0 when added together
    default_configuration: u8, // If this is not zero then configuration_table should be
    // ignored and a default configuration should be loaded instead
    features: u32, // If bit 7 is then the IMCR is present and PIC mode is being used, otherwise
                   // virtual wire mode is; all other bits are reserved
}

#[repr(C, packed)]
struct ConfigurationTable {
    signature: [char; 4], // "PCMP"
    length: u32,
    mp_specification_revision: u8,
    checksum: u8, // Again, the byte should be all bytes in the table add up to 0
    oem_id: [char; 8],
    product_id: [char; 12],
    oem_table: u32,
    oem_table_size: u16,
    entry_count: u16, // This value represents how many entries are following this table
    lapic_address: u32, // This is the memory mapped address of the local APICs
    extended_table_length: u16,
    extended_table_checksum: u8,
    _reserved: u8,
}

#[repr(C, packed)]
struct ProcessorEntry {
    proc_type: u8, // Always 0
    local_apic_id: u8,
    local_apic_version: u8,
    flags: u8, // If bit 0 is clear then the processor must be ignored
    // If bit 1 is set then the processor is the bootstrap processor
    signature: u32,
    feature_flags: u32,
    _reserved: u64,
}

#[repr(C, packed)]
struct OIApicEntry {
    apci_type: u8, // Always 2
    id: u8,
    version: u8,
    flags: u8,    // If bit 0 is set then the entry should be ignored
    address: u32, // The memory mapped address of the IO APIC is memory
}
