use bit_field::BitField;
use core::convert::TryFrom;
use core::fmt::{self, Debug, Formatter};

pub struct PhysicalAddress(u64);

#[derive(Debug)]
pub struct PhysicalAddressInvalid(u64);

// we can only use the lower 52 bits, the top 12 need to be zero

impl PhysicalAddress {
    pub fn new(address: u64) -> Self {
        Self::try_new(address).expect("Invalid Physical Address")
    }

    fn try_new(address: u64) -> Result<Self, PhysicalAddressInvalid> {
        match address.get_bits(52..64) {
            0 => Ok(Self(address)), // address is valid
            other => Err(PhysicalAddressInvalid(other)),
        }
    }

    pub fn truncate_new(address: u64) -> Self {
        Self(address % (1 << 52))
    }
}

impl TryFrom<u64> for PhysicalAddress {
    type Error = PhysicalAddressInvalid;
    fn try_from(value: u64) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl Debug for PhysicalAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_tuple("VirtualAddress")
            .field(&format_args!("{:#x}", self.0))
            .finish()
    }
}
