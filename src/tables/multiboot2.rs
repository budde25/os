use crate::address::phys::PhysicalAddress;
use crate::tables::rsdp::{RSDPV1, RSDPV2};
use core::fmt::Debug;

/// A struct for parsing multiboot2 boot info
#[derive(Debug)]
pub struct Multiboot {
    start_address: PhysicalAddress,
    multiboot_table: &'static MultibootTable,
    index: u32,
    boot_command_line: Option<&'static BootCommandLine>,
    boot_loader_name: Option<&'static BootLoaderName>,
    modules: Option<&'static Modules>, // FIXME allow for an arbitary amount of modules
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
            let item = start + self.index as u64;
            self.index += 8; // increment the tag size

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
                StructType::RSDPV1 => {
                    self.rsdp_v1 = {
                        let fixed = start + self.index as u64;
                        unsafe { Some(&*fixed.as_ptr::<RSDPV1>()) }
                    }
                }
                StructType::RSDPV2 => {
                    self.rsdp_v2 = {
                        let fixed = start + self.index as u64;
                        unsafe { Some(&*fixed.as_ptr::<RSDPV2>()) }
                    }
                }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    EFIError = 18,   // This tag indicates ExitBootServices wasn’t called
    EFI32Image = 19, // 32-bit image handle pointer
    EFI64Image = 20, // 64-bit image handle pointer
    ImageLoaderBase = 21,
}

// The header of the multiboot table
#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct MultibootTable {
    total_size: u32,
    _reserved: u32,
}

impl Debug for MultibootTable {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let size = self.total_size;
        f.debug_struct("MultibootTable")
            .field("total_size", &size)
            .finish()
    }
}

/// A tag type, which represents the following structure and its size
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct Tag {
    r#type: StructType,
    size: u32,
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct BootCommandLine {
    r#type: StructType,
    size: u32,
    string: [u8; 0], // slice of length size - 8
}

impl BootCommandLine {
    pub fn string(&self) -> &'static str {
        let string_offset = 8;
        let ptr = self as *const BootCommandLine as *const u8;
        let slice =
            unsafe { core::slice::from_raw_parts(ptr.add(string_offset), self.size as usize - 8) };
        unsafe { core::str::from_utf8_unchecked(slice) }
    }
}

impl Debug for BootCommandLine {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("BootCommandLine")
            .field("string", &self.string())
            .finish()
    }
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct BootLoaderName {
    r#type: StructType,
    size: u32,
    string: [u8; 0], // slice of length size - 8
}

impl BootLoaderName {
    pub fn string(&self) -> &'static str {
        let string_offset = 8;
        let ptr = self as *const BootLoaderName as *const u8;
        let slice =
            unsafe { core::slice::from_raw_parts(ptr.add(string_offset), self.size as usize - 8) };
        unsafe { core::str::from_utf8_unchecked(slice) }
    }
}

impl Debug for BootLoaderName {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("BootLoaderName")
            .field("string", &self.string())
            .finish()
    }
}

/// This tag indicates to the kernel what boot module was loaded along with the kernel image, and where it can be found.
/// One tag appears per module. This tag type may appear multiple times.
#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct Modules {
    r#type: StructType,
    size: u32,
    mod_start: u32,
    mod_end: u32,
    string: [u8; 0], // slice of length size - 8
}

impl Modules {
    pub fn string(&self) -> &'static str {
        let string_offset = 16;
        let ptr = self as *const Modules as *const u8;
        let slice =
            unsafe { core::slice::from_raw_parts(ptr.add(string_offset), self.size as usize - 8) };
        unsafe { core::str::from_utf8_unchecked(slice) }
    }

    pub fn mod_start(&self) -> PhysicalAddress {
        PhysicalAddress::new(self.mod_start.into())
    }

    pub fn mod_end(self) -> PhysicalAddress {
        PhysicalAddress::new(self.mod_end.into())
    }
}

impl Debug for Modules {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Modules")
            .field("mod_start", &self.mod_start())
            .field("mod_end", &self.mod_end())
            .field("string", &self.string)
            .finish()
    }
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct BasicMemoryInfo {
    r#type: StructType,
    size: u32,
    upper: u32,
    lower: u32,
}

impl Debug for BasicMemoryInfo {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let upper = self.upper;
        let lower = self.lower;
        f.debug_struct("BasicMemoryInfo")
            .field("upper", &upper)
            .field("lower", &lower)
            .finish()
    }
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct BIOSBootDevice {
    r#type: StructType,
    size: u32,
    biosdev: u32,
    partition: u32,
    sub_partition: u32,
}

impl Debug for BIOSBootDevice {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let biosdev = self.biosdev;
        let partition = self.partition;
        let sub_partition = self.sub_partition;
        f.debug_struct("BIOSBootDevice")
            .field("biosdev", &biosdev)
            .field("partition", &partition)
            .field("sub_partition", &sub_partition)
            .finish()
    }
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct VBEInfo {
    r#type: StructType,
    size: u32,
    mode: u16,
    interface_seg: u16,
    interface_offset: u16,
    interface_len: u16,
    control_info: [u8; 512],
    mode_info: [u8; 256],
}

impl Debug for VBEInfo {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mode = self.mode;
        let interface_seg = self.interface_seg;
        let interface_offset = self.interface_offset;
        let interface_len = self.interface_len;
        let control_info = self.control_info;
        let mode_info = self.mode_info;
        f.debug_struct("VBEInfo")
            .field("mode", &mode)
            .field("interface_seg", &interface_seg)
            .field("interface_offset", &interface_offset)
            .field("interface_len", &interface_len)
            .field("control_info", &control_info)
            .field("mode_info", &mode_info)
            .finish()
    }
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct FrameBufferInfo {
    r#type: StructType,
    size: u32,
    address: u64,
    pitch: u32,
    width: u32,
    height: u32,
    bpp: u8,
    framebuffer_type: FrameBufferType,
    _reserved: u8,
    color_info: [u8; 0], // color info data defined in structs below
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FrameBufferType {
    Indexed = 0,
    DirectRgb = 1,
}

impl FrameBufferInfo {
    /// The address to the framebuffer
    fn address(&self) -> PhysicalAddress {
        PhysicalAddress::new(self.address)
    }

    fn pitch(&self) -> u32 {
        self.pitch
    }

    fn width(&self) -> u32 {
        self.pitch
    }

    fn height(&self) -> u32 {
        self.height
    }

    /// bits per pixel
    fn bpp(&self) -> u8 {
        self.bpp
    }

    fn framebuffer_type(&self) -> FrameBufferType {
        self.framebuffer_type
    }

    fn color_direct_rgb(&self) -> Option<&'static ColorDirectRgb> {
        if self.framebuffer_type != FrameBufferType::DirectRgb {
            return None;
        }
        let offset = 31;
        let ptr = self as *const FrameBufferInfo as *const u8;
        Some(unsafe { &*(ptr.add(offset) as *const ColorDirectRgb) })
    }

    fn color_indexed(&self) -> Option<&'static ColorIndexed> {
        if self.framebuffer_type != FrameBufferType::Indexed {
            return None;
        }
        let offset = 31;
        let ptr = self as *const FrameBufferInfo as *const u8;
        Some(unsafe { &*(ptr.add(offset) as *const ColorIndexed) })
    }
}

impl Debug for FrameBufferInfo {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("FrameBufferInfo")
            .field("address", &self.address())
            .field("pitch", &self.pitch())
            .field("width", &self.width())
            .field("height", &self.height())
            .field("bpp", &self.bpp())
            .field("type", &self.framebuffer_type())
            .finish()
    }
}

// types for framebuffer

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
struct ColorDirectRgb {
    red_field_position: u8,
    red_mask_size: u8,
    green_field_position: u8,
    green_mask_size: u8,
    blue_field_position: u8,
    blue_mask_size: u8,
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
struct ColorIndexed {
    num_colors: u32,
}

impl ColorIndexed {
    fn pallets(&self) -> &'static [Pallet] {
        let offset = 4;
        let ptr = self as *const ColorIndexed as *const u8;
        let ptr = unsafe { ptr.add(offset) };
        let ptr = ptr as *const Pallet;
        unsafe { core::slice::from_raw_parts(ptr.add(offset), self.num_colors as usize) }
    }
}

impl Debug for ColorIndexed {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let num_colors = self.num_colors;
        f.debug_struct("ColorIndexed")
            .field("num_colors", &num_colors)
            .field("pallets", &self.pallets())
            .finish()
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
struct Pallet {
    red_value: u8,
    green_value: u8,
    blue_value: u8,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct MemoryMap {
    r#type: StructType,
    size: u32,
    entry_size: u32,
    entry_version: u32,
    entries: [u128; 0], // FIXME allow for data
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct ElfSymbols {
    r#type: StructType,
    size: u32,
    num: u16,
    entsize: u16,
    shndx: u16,
    _reserved: u16,
    section_headers: [u8; 0], // FIXME allow for data
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct APMTable {
    r#type: StructType,
    size: u32,
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
    r#type: StructType,
    size: u32,
    pointer: u32,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct EFI64Table {
    r#type: StructType,
    size: u32,
    pointer: u64,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct SMBIOSTables {
    r#type: StructType,
    size: u32,
    major: u8,
    minor: u8,
    _reserved: [u8; 6],
    smbios_tables: [u8; 0], // FIXME allow for data
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct NetworkInfo {
    r#type: StructType,
    size: u32,
    dhcp_ack: [u8; 0], // FIXME allow for data
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct EFIMemoryMap {
    r#type: StructType,
    size: u32,
    descriptor_size: u32,
    descriptor_version: u32,
    entries: [u8; 0], // FIXME allow for data
}

#[derive(Debug, Clone, Copy)]
pub struct EFIError {
    r#type: StructType,
    size: u32,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct EFI32Image {
    r#type: StructType,
    size: u32,
    pointer: u32,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct EFI64Image {
    r#type: StructType,
    size: u32,
    pointer: u64,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct ImageLoaderBase {
    r#type: StructType,
    size: u32,
    load_base_addr: u32,
}
