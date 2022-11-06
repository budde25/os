use spin::Lazy;

use multiboot2::MultibootInfo;
use x86_64::tables::acpi::Acpi;
use x86_64::tables::madt::MultiAPIC;

pub static MULTIBOOT_INFO: Lazy<MultibootInfo> = Lazy::new(|| {
    extern "C" {
        static multiboot_info_ptr: u32;
    }
    let mbinfo = unsafe { MultibootInfo::new(multiboot_info_ptr as usize) }
        .expect("There should be valid multiboot info table");
    mbinfo
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