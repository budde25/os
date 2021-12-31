pub mod acpi;
pub mod madt;
pub mod multiboot2;
mod multiproc;

use acpi::Acpi;
use madt::MultiAPIC;
use multiboot2::Multiboot;
use spin::Lazy;

pub static MULTIBOOT: Lazy<Multiboot> = Lazy::new(|| {
    let mut table = Multiboot::new();
    table.init();
    table
});

pub static ACPI_TABLE: Lazy<Acpi> = Lazy::new(|| {
    let rsdp = MULTIBOOT.rsdp_v1.unwrap();
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
