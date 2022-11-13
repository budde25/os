use crate::PhysicalAddress;
use core::fmt::Debug;
use core::{mem, slice, str};

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct RsdpV1 {
    signature: [u8; 8],
    checksum: u8,
    oem_id: [u8; 6],
    revision: u8,
    rsdt_address: u32,
}

impl RsdpV1 {
    pub fn rsdt_address(&self) -> PhysicalAddress {
        PhysicalAddress::new(self.rsdt_address.into())
    }

    /// Validation using the checksum
    pub fn is_valid(&self) -> bool {
        let bytes =
            unsafe { slice::from_raw_parts(self as *const _ as *const u8, mem::size_of::<Self>()) };
        bytes.iter().fold(0u8, |acc, val| acc.wrapping_add(*val)) == 0
    }
}

impl Debug for RsdpV1 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let signature = str::from_utf8(&self.signature).unwrap();
        let oem_id = str::from_utf8(&self.oem_id).unwrap();
        f.debug_struct("RsdpV1")
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
pub struct RsdpV2 {
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

impl RsdpV2 {
    pub fn rsdt_address(&self) -> PhysicalAddress {
        PhysicalAddress::new(self.rsdt_address.into())
    }

    pub fn xsdt_address(&self) -> PhysicalAddress {
        PhysicalAddress::new(self.xsdt_address)
    }

    /// Validation using the checksum
    pub fn is_valid(&self) -> bool {
        let bytes =
            unsafe { slice::from_raw_parts(self as *const _ as *const u8, mem::size_of::<Self>()) };
        bytes.iter().fold(0u8, |acc, val| acc.wrapping_add(*val)) == 0
    }
}

impl Debug for RsdpV2 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let signature = str::from_utf8(&self.signature).unwrap();
        let oem_id = str::from_utf8(&self.oem_id).unwrap();
        let length = self.length; // fix unaligned ref
        f.debug_struct("RsdpV2")
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
