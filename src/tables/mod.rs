pub mod acpi;
mod madt;
pub mod multiboot2;
mod multiproc;

use acpi::ACPI;
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
        ACPI::new(rsdp.rsdt_address())
    };
}
