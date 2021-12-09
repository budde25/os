use crate::address::phys::PhysicalAddress;
use crate::tables::rsdp::{RSDPV1, RSDPV2};
use core::fmt::Debug;

#[derive(Debug)]
pub struct Multiboot {
    start_address: PhysicalAddress,
    multiboot_table: &'static MultibootTable,
    index: u32,
    boot_command_line: Option<&'static BootCommandLine>,
    boot_loader_name: Option<&'static BootLoaderName>,
    modules: Option<&'static Modules>,
    basic_memory_info: Option<&'static BasicMemoryInfo>,
    bios_boot_device: Option<&'static BIOSBootDevice>,
    memory_map: Option<&'static MemoryMap>,
    vbe_info: Option<&'static VBEInfo>,
    frame_buffer_info: Option<&'static FrameBufferInfo>,
    elf_symbols: Option<&'static ElfSymbols>,
    apm_table: Option<&'static APMTable>,
    efi_32_table: Option<&'static EFI32Table>,
    efi_64_table: Option<&'static EFI64Table>,
    sm_bios_tables: Option<&'static SMBIOSTables>,
    rsdp_v1: Option<&'static RSDPV1>,
    rsdp_v2: Option<&'static RSDPV2>,
    network_info: Option<&'static NetworkInfo>,
    efi_memory_map: Option<&'static EFIMemoryMap>,
    efi_error: Option<&'static EFIError>,
    efi_32_image: Option<&'static EFI32Image>,
    efi_64_image: Option<&'static EFI64Image>,
    image_loader_base: Option<&'static ImageLoaderBase>,
}

impl Multiboot {
    pub fn new() -> Self {
        extern "C" {
            static multiboot_info_ptr: u32;
        }

        let addr = unsafe { PhysicalAddress::new(multiboot_info_ptr.into()) };

        Self {
            start_address: addr,
            multiboot_table: unsafe { &*addr.as_ptr::<MultibootTable>() },
            index: 0,
            boot_command_line: None,
            boot_loader_name: None,
            memory_map: None,
            apm_table: None,
            elf_symbols: None,
            efi_memory_map: None,
            modules: None,
            basic_memory_info: None,
            bios_boot_device: None,
            vbe_info: None,
            frame_buffer_info: None,
            efi_32_table: None,
            efi_64_table: None,
            sm_bios_tables: None,
            rsdp_v1: None,
            rsdp_v2: None,
            network_info: None,
            efi_error: None,
            efi_32_image: None,
            efi_64_image: None,
            image_loader_base: None,
        }
    }

    pub fn init(&mut self) {
        self.index = 8;
        let start = self.start_address;
        while self.index <= self.total_size() {
            let tag_addr = start + self.index as u64;
            let tag = unsafe { *tag_addr.as_ptr::<Tag>() };
            let item_size = tag.size - 8; // minus 8 since its the size of the tag
            self.index += 8; // increment the tag size
            let item = start + self.index as u64;

            match tag.r#type {
                StructType::Terminate => return,
                StructType::BootCommandLine => {
                    self.boot_command_line = unsafe { Some(&*item.as_ptr::<BootCommandLine>()) }
                }
                StructType::BootLoaderName => {
                    self.boot_loader_name = unsafe { Some(&*item.as_ptr::<BootLoaderName>()) }
                }
                StructType::Modules => self.modules = unsafe { Some(&*item.as_ptr::<Modules>()) },
                StructType::BasicMemoryInfo => {
                    self.basic_memory_info = unsafe { Some(&*item.as_ptr::<BasicMemoryInfo>()) }
                }
                StructType::BIOSBootDevice => {
                    self.bios_boot_device = unsafe { Some(&*item.as_ptr::<BIOSBootDevice>()) }
                }
                StructType::MemoryMap => {
                    self.memory_map = unsafe { Some(&*item.as_ptr::<MemoryMap>()) }
                }
                StructType::VBEInfo => self.vbe_info = unsafe { Some(&*item.as_ptr::<VBEInfo>()) },
                StructType::FrameBufferInfo => {
                    self.frame_buffer_info = unsafe { Some(&*item.as_ptr::<FrameBufferInfo>()) }
                }
                StructType::ELFSymbols => {
                    self.elf_symbols = unsafe { Some(&*item.as_ptr::<ElfSymbols>()) }
                }
                StructType::APMTable => {
                    self.apm_table = unsafe { Some(&*item.as_ptr::<APMTable>()) }
                }
                StructType::EFI32Table => {
                    self.efi_32_table = unsafe { Some(&*item.as_ptr::<EFI32Table>()) }
                }
                StructType::EFI64Table => {
                    self.efi_64_table = unsafe { Some(&*item.as_ptr::<EFI64Table>()) }
                }
                StructType::SMBIOSTables => {
                    self.sm_bios_tables = unsafe { Some(&*item.as_ptr::<SMBIOSTables>()) }
                }
                StructType::RSDPV1 => self.rsdp_v1 = unsafe { Some(&*item.as_ptr::<RSDPV1>()) },
                StructType::RSDPV2 => self.rsdp_v2 = unsafe { Some(&*item.as_ptr::<RSDPV2>()) },
                StructType::NetworkInfo => {
                    self.network_info = unsafe { Some(&*item.as_ptr::<NetworkInfo>()) }
                }
                StructType::EFIMemoryMap => {
                    self.efi_memory_map = unsafe { Some(&*item.as_ptr::<EFIMemoryMap>()) }
                }
                StructType::EFIError => {
                    self.efi_error = unsafe { Some(&*item.as_ptr::<EFIError>()) }
                }
                StructType::EFI32Image => {
                    self.efi_32_image = unsafe { Some(&*item.as_ptr::<EFI32Image>()) }
                }
                StructType::EFI64Image => {
                    self.efi_64_image = unsafe { Some(&*item.as_ptr::<EFI64Image>()) }
                }
                StructType::ImageLoaderBase => {
                    self.image_loader_base = unsafe { Some(&*item.as_ptr::<ImageLoaderBase>()) }
                }
            }

            self.index += item_size;
            let remainder = self.index % 8;
            if remainder != 0 {
                self.index += 8 - remainder;
            }
        }
    }

    pub fn total_size(&self) -> u32 {
        self.multiboot_table.total_size
    }
}

impl Default for Multiboot {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub enum StructType {
    Terminate = 0,
    BootCommandLine = 1,
    BootLoaderName = 2,
    Modules = 3,
    BasicMemoryInfo = 4,
    BIOSBootDevice = 5,
    MemoryMap = 6,
    VBEInfo = 7,
    FrameBufferInfo = 8,
    ELFSymbols = 9,
    APMTable = 10,
    EFI32Table = 11,
    EFI64Table = 12,
    SMBIOSTables = 13,
    RSDPV1 = 14,
    RSDPV2 = 15,
    NetworkInfo = 16,
    EFIMemoryMap = 17,
    EFIError = 18,        // This tag indicates ExitBootServices wasn’t called
    EFI32Image = 19,      // 32-bit image handle pointer
    EFI64Image = 20,      // 64-bit image handle pointer
    ImageLoaderBase = 21, // physical addr
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct MultibootTable {
    total_size: u32,
    _reserved: u32,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct Tag {
    r#type: StructType,
    size: u32,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct BootCommandLine {
    string: [char; 0], // FIXME allow for data
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct BootLoaderName {
    string: [char; 0], // FIXME allow for data
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct Modules {
    mod_start: u32,
    mod_end: u32,
    string: [char; 0], // FIXME allow for data
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct BasicMemoryInfo {
    upper: u32,
    lower: u32,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct BIOSBootDevice {
    biosdev: u32,
    partition: u32,
    sub_partition: u32,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct VBEInfo {
    vbe_mode: u16,
    vbe_interface_seg: u16,
    vbe_interface_offset: u16,
    vbe_interface_len: u16,
    vbe_control_info: [u8; 512],
    vbe_mode_info: [u8; 256],
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct FrameBufferInfo {
    framebuffer_addr: u64,
    framebuffer_pitch: u32,
    framebuffer_width: u32,
    framebuffer_height: u32,
    framebuffer_bpp: u8,
    framebuffer_type: u8,
    _reserved: u8,
    color_info: [u8; 0], // FIXME allow for data
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct MemoryMap {
    entry_size: u32,
    entry_version: u32,
    entries: [u128; 0], // FIXME allow for data
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct ElfSymbols {
    num: u16,
    entsize: u16,
    shndx: u16,
    _reserved: u16,
    section_headers: [u8; 0], // FIXME allow for data
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct APMTable {
    version: u16,
    cseg: u16,
    offset: u32,
    cseg_16: u16,
    dseg: u16,
    flags: u16,
    cseg_len: u16,
    cseg_16_len: u16,
    dseg_len: u16,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct EFI32Table {
    pointer: u32,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct EFI64Table {
    pointer: u64,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct SMBIOSTables {
    major: u8,
    minor: u8,
    _reserved: [u8; 6],
    smbios_tables: [u8; 0], // FIXME allow for data
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct NetworkInfo {
    dhcp_ack: [u8; 0], // FIXME allow for data
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct EFIMemoryMap {
    descriptor_size: u32,
    descriptor_version: u32,
    entries: [u8; 0], // FIXME allow for data
}

#[derive(Debug, Clone, Copy)]
pub struct EFIError;

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct EFI32Image {
    pointer: u32,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct EFI64Image {
    pointer: u64,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct ImageLoaderBase {
    load_base_addr: u32,
}
