/// Fat 32 implemenation

#[repr(C, packed)]
struct BiosParameterBlock {
    jmp_boot: [u8; 3],
    oem_name: u64,
    bytes_per_sector: u16,
    sectors_per_cluster: u8,
    reserved_sector_count: u16,
    num_fats: u8,
    root_entry_count: u16,
    _reserved_2: u16, // must be zero on fat32
    media: u8,
    _reserved_3: u16, // must be zero on fat32
    sectors_per_track: u16,
    num_heads: u16,
    hidden_sectors: u32,
    total_sectors: u32,

    /// Extended boot record, should be right after BiosParameterBlcok
    fatsz: u32, // the count of sectors occupied by ONE FAT
    ext_flags: u16,
    version_minor: u8,
    version_high: u8,
    root_cluster: u32,
    fs_info: u16,
    boot_sector: u16, // should be 6
    _reserved_4: [u8; 12],
    drive_num: u8,
    _reserved_5: u8,
    boot_signature: u8,
    volume_id: u32,
    volume_label: [u8; 11],
    file_system_type: [u8; 8], // should be the cstring 'FAT32   '
}

impl BiosParameterBlock {
    fn first_data_sector(&self) -> u32 {
        let root_dir_sectors = 0; // always 0 for fat32
        self.reserved_sector_count as u32 + (self.num_fats as u32 * self.fatsz) + root_dir_sectors
    }

    fn first_sector_of_cluster(&self, cluster: u32) -> u32 {
        ((cluster - 2) * self.sectors_per_cluster as u32) + self.first_data_sector()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test_case]
    fn test_bpb_size() {
        use core::mem::size_of;

        assert_eq!(size_of::<BiosParameterBlock>(), 90);
    }
}
