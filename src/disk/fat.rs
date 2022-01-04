/// Fat 32 implementation

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct BiosParameterBlock {
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

    /// Extended boot record, should be right after BiosParameterBlock
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

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]

struct FsInfo {
    lead_signature: u32,    // should always be 0x41615252
    _reserved_1: [u8; 480], // should be zero
    struct_signature: u32,  // should always be 0x61417272
    free_count: u32,        // last known free cluster, if 0xFFFFFFFF must be computed
    next_free: u32, // hint cluster number where we should look for enter free cluster, if 0xFFFFFFFF we should start at 2
    trail_signature: u32, // should always ve 0xAA550000
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
struct DirectoryEntry {
    name: [u8; 11],
    attributes: u8,
    _reserved: u8,
    create_time_tenth: u8,
    create_time: u16,
    create_date: u16,
    last_access_date: u16,
    first_cluster_hi: u16,
    write_time: u16,
    write_date: u16,
    first_cluster_lo: u16,
    file_size: u32,
}

impl BiosParameterBlock {
    const fn first_data_sector(&self) -> u32 {
        let root_dir_sectors = 0; // always 0 for fat32
        self.reserved_sector_count as u32 + (self.num_fats as u32 * self.fatsz) + root_dir_sectors
    }

    const fn first_sector_of_cluster(&self, cluster: u32) -> u32 {
        ((cluster - 2) * self.sectors_per_cluster as u32) + self.first_data_sector()
    }

    const fn data_sectors(&self) -> u32 {
        self.total_sectors - self.first_data_sector()
    }

    const fn count_of_clusters(&self) -> u32 {
        self.data_sectors() / self.sectors_per_cluster as u32
    }

    const fn this_fat_sec_num(&self, cluster: u32) -> u32 {
        let fat_offset = cluster * 4;
        self.reserved_sector_count as u32 + (fat_offset / self.bytes_per_sector as u32)
    }

    const fn this_fat_ent_offset(&self, cluster: u32) -> u32 {
        let fat_offset = cluster * 4;
        fat_offset & self.bytes_per_sector as u32
    }

    pub const fn is_fat32(&self) -> bool {
        let coc = self.count_of_clusters();
        if coc < 4085 {
            // volume is FAT12
            return false;
        } else if coc < 65525 {
            // volume is FAT16
            return false;
        }
        // volume is FAT32
        true
    }

    pub fn volume_label(&self) -> &'static str {
        let ptr = &self.volume_label as *const _ as *const u8;
        let slice = unsafe { core::slice::from_raw_parts(ptr, 11) };
        unsafe { core::str::from_utf8_unchecked(slice) }
    }

    pub fn file_system_type(&self) -> &'static str {
        let ptr = &self.file_system_type as *const _ as *const u8;
        let slice = unsafe { core::slice::from_raw_parts(ptr, 8) };
        unsafe { core::str::from_utf8_unchecked(slice) }
    }

    pub fn oem_name(&self) -> &'static str {
        let oem_name = self.oem_name;
        let ptr = &oem_name as *const _ as *const u8;
        let slice = unsafe { core::slice::from_raw_parts(ptr, 8) };
        unsafe { core::str::from_utf8_unchecked(slice) }
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
