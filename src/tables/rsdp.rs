use core::fmt::Debug;
use core::str;

use crate::address::phys::PhysicalAddress;

const RSDP_SIGNATURE: [char; 8] = ['R', 'S', 'D', ' ', 'P', 'T', 'R', ' '];

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct RSDPV1 {
    signature: [u8; 8],
    checksum: u8,
    oem_id: [u8; 6],
    revision: u8,
    rsdt_address: u32,
}

impl RSDPV1 {
    pub fn rsdt_address(&self) -> PhysicalAddress {
        PhysicalAddress::new(self.rsdt_address.into())
    }
}

impl Debug for RSDPV1 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let signature = str::from_utf8(&self.signature).unwrap();
        let oem_id = str::from_utf8(&self.oem_id).unwrap();
        f.debug_struct("RESDV1")
            .field("signature", &signature)
            .field("checksum", &self.checksum)
            .field("oem_id", &oem_id)
            .field("revision", &self.revision)
            .field("rsdt_address", &self.rsdt_address())
            .finish()
    }
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct RSDPV2 {
    signature: [u8; 8],
    checksum: u8,
    oem_id: [u8; 6],
    revision: u8,
    rsdt_address: u32,
    // only if revision 2
    length: u32,
    xsdt_address: u64,
    extended_checksum: u8,
    _reserved: [u8; 3],
}

impl RSDPV2 {
    pub fn rsdt_address(&self) -> PhysicalAddress {
        PhysicalAddress::new(self.rsdt_address.into())
    }

    pub fn xsdt_address(&self) -> PhysicalAddress {
        PhysicalAddress::new(self.xsdt_address)
    }
}

impl Debug for RSDPV2 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let signature = str::from_utf8(&self.signature).unwrap();
        let oem_id = str::from_utf8(&self.oem_id).unwrap();
        let length = self.length; // fix unaligned ref
        f.debug_struct("RESDV2")
            .field("signature", &signature)
            .field("checksum", &self.checksum)
            .field("oem_id", &oem_id)
            .field("revision", &self.revision)
            .field("rsdt_address", &self.rsdt_address())
            .field("length", &length)
            .field("xsdt_address", &self.xsdt_address())
            .field("extended_checksum", &self.extended_checksum)
            .finish()
    }
}
