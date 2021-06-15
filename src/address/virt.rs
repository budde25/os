use bit_field::BitField;
use core::fmt::{self, Debug, Formatter};

/// Much of the code in this section is used from Phil's excellent x86_64
/// https://github.com/rust-osdev/x86_64/blob/master/src/addr.rs

/// Virtual address space
/// https://en.wikipedia.org/wiki/X86-64#Virtual_address_space_details
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct VirtualAddress(u64);

#[derive(Debug)]
pub struct VirtualAddressInvalid(u64);

impl VirtualAddress {
    pub fn new(address: u64) -> Self {
        Self::try_new(address).expect("Invalid Virtual Address")
    }

    pub fn try_new(address: u64) -> Result<Self, VirtualAddressInvalid> {
        match address.get_bits(47..64) {
            0 | 0x1ffff => Ok(Self(address)),     // address is canonical
            1 => Ok(Self::truncate_new(address)), // address needs sign extension
            other => Err(VirtualAddressInvalid(other)),
        }
    }

    pub fn truncate_new(address: u64) -> Self {
        Self(((address << 16) as i64 >> 16) as u64)
    }
}

impl Debug for VirtualAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_tuple("VirtualAddress")
            .field(&format_args!("{:#x}", self.0))
            .finish()
    }
}
