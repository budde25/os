use super::Tag;

use x86_64::tables::rsdp::RsdpV1;
use x86_64::tables::rsdp::RsdpV2;

#[derive(Debug, Clone, Copy)]
pub struct RsdpV1Tag {
    tag: Tag,
    table: RsdpV1,
}

impl RsdpV1Tag {
    pub fn table(&self) -> RsdpV1 {
        self.table
    }

    pub fn tag(&self) -> Tag {
        self.tag
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RsdpV2Tag {
    tag: Tag,
    table: RsdpV2,
}

impl RsdpV2Tag {
    pub fn table(&self) -> RsdpV2 {
        self.table
    }

    pub fn tag(&self) -> Tag {
        self.tag
    }
}
