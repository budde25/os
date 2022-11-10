use spin::Lazy;

use multiboot2::{MultiBoot2Header, MultibootInfo};
use x86_64::tables::acpi::Acpi;
use x86_64::tables::madt::MultiAPIC;

/// Puts the multiboot2 header at section .multiboot allowing for booting for bootloaders such as grub2
#[no_mangle]
#[link_section = ".multiboot"]
pub static MULTIBOOT2_HEADER: MultiBoot2Header = MultiBoot2Header::new();

/// Global variable set by the assembly code and loaded with pointer to the multiboot info table
/// # Saftey
/// * This function is never set after the assembly code inits it
#[no_mangle]
pub static mut multiboot_info_ptr: u32 = 0;

pub static MULTIBOOT_INFO: Lazy<MultibootInfo> = Lazy::new(|| {
    unsafe { MultibootInfo::new(multiboot_info_ptr as usize) }
        .expect("There should be valid multiboot info table")
});

pub static ACPI_TABLE: Lazy<Acpi> = Lazy::new(|| {
    let rsdp = MULTIBOOT_INFO.rsdp_v1().unwrap().table();
    let mut acpi = Acpi::new(rsdp.rsdt_address());
    acpi.init();
    acpi
});

pub static MADT_TABLE: Lazy<MultiAPIC> = Lazy::new(|| {
    let madt_ptr = ACPI_TABLE.madt_ptr.unwrap();
    let mut madt = MultiAPIC::new(madt_ptr);
    madt.init();
    madt
});
