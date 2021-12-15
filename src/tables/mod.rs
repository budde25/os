pub mod acpi;
pub mod madt;
pub mod multiboot2;
mod multiproc;

use acpi::ACPI;
use madt::MultiAPIC;
use multiboot2::Multiboot;

use lazy_static::lazy_static;

lazy_static! {
    pub static ref MULTIBOOT: Multiboot = {
        let mut table = Multiboot::new();
        table.init();
        table
    };
    pub static ref ACPI_TABLE: ACPI = {
        let rsdp = MULTIBOOT.rsdp_v1.unwrap();
        let mut acpi = ACPI::new(rsdp.rsdt_address());
        acpi.init();
        acpi
    };
    pub static ref MADT_TABLE: MultiAPIC = {
        let madt_ptr = ACPI_TABLE.madt_ptr.unwrap();
        let mut madt = MultiAPIC::new(madt_ptr);
        madt.init();
        madt
    };
}
