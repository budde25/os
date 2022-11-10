use super::Tag;

use x86_64::tables::acpi::RSDPV1;
use x86_64::tables::acpi::RSDPV2;

#[derive(Debug, Clone, Copy)]
pub struct RsdpV1 {
    tag: Tag,
    table: RSDPV1,
}

impl RsdpV1 {
    pub fn table(&self) -> RSDPV1 {
        self.table
    }

    pub fn tag(&self) -> Tag {
        self.tag
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RsdpV2 {
    tag: Tag,
    table: RSDPV2,
}

impl RsdpV2 {
    pub fn table(&self) -> RSDPV2 {
        self.table
    }

    pub fn tag(&self) -> Tag {
        self.tag
    }
}
