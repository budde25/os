use crate::paging::page_table::{PageOffset, PageTableIndex};

use super::{align_down, align_up, phys::PhysicalAddress};

use bit_field::BitField;
use core::convert::{From, TryFrom};
use core::fmt::{self, Debug, Formatter};
use core::ops::{Add, AddAssign, Sub, SubAssign};
use derive_more::{Binary, Display, Into, LowerHex, UpperHex};

/// Much of the code in this section is used from Phil's excellent x86_64
/// https://github.com/rust-osdev/x86_64/blob/master/src/addr.rs

/// Virtual address space
/// https://en.wikipedia.org/wiki/X86-64#Virtual_address_space_details
#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Into, Display, UpperHex, LowerHex, Binary,
)]
#[display(fmt = "V_0x{:X}", _0)]
#[repr(transparent)]
pub struct VirtualAddress(u64);

#[derive(Debug)]
pub struct VirtualAddressInvalid(u64);

impl VirtualAddress {
    pub fn new(address: u64) -> Self {
        Self::try_new(address).expect("Invalid Virtual Address")
    }

    fn try_new(address: u64) -> Result<Self, VirtualAddressInvalid> {
        match address.get_bits(47..64) {
            0 | 0x1ffff => Ok(Self(address)),     // address is canonical
            1 => Ok(Self::truncate_new(address)), // address needs sign extension
            _ => Err(VirtualAddressInvalid(address)),
        }
    }

    pub const fn truncate_new(address: u64) -> Self {
        Self(((address << 16) as i64 >> 16) as u64)
    }

    pub const fn is_null(&self) -> bool {
        self.0 == 0
    }

    #[must_use]
    pub fn align_down<U>(self, align: U) -> Self
    where
        U: Into<u64>,
    {
        Self(align_down(self.0, align.into()))
    }

    #[must_use]
    pub fn align_up<U>(self, align: U) -> Self
    where
        U: Into<u64>,
    {
        Self(align_up(self.0, align.into()))
    }

    pub fn is_aligned<U>(self, align: U) -> bool
    where
        U: Into<u64>,
    {
        self.align_down(align) == self
    }

    pub fn as_ptr<T>(&self) -> *const T {
        self.0 as *const T
    }

    pub fn as_mut_ptr<T>(&self) -> *mut T {
        self.as_ptr::<T>() as *mut T
    }

    /// Returns the 12-bit page offset of this virtual address.
    pub const fn page_offset(self) -> PageOffset {
        PageOffset::new_truncate(self.0 as u16)
    }

    /// Returns the 9-bit level 1 page table index.
    pub const fn p1_index(self) -> PageTableIndex {
        PageTableIndex::new_truncate((self.0 >> 12) as u16)
    }

    /// Returns the 9-bit level 2 page table index.
    pub const fn p2_index(self) -> PageTableIndex {
        PageTableIndex::new_truncate((self.0 >> 12 >> 9) as u16)
    }

    /// Returns the 9-bit level 3 page table index.
    pub const fn p3_index(self) -> PageTableIndex {
        PageTableIndex::new_truncate((self.0 >> 12 >> 9 >> 9) as u16)
    }

    /// Returns the 9-bit level 4 page table index.
    pub const fn p4_index(self) -> PageTableIndex {
        PageTableIndex::new_truncate((self.0 >> 12 >> 9 >> 9 >> 9) as u16)
    }
}

impl TryFrom<usize> for VirtualAddress {
    type Error = VirtualAddressInvalid;
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Self::try_new(value as u64)
    }
}

impl TryFrom<u64> for VirtualAddress {
    type Error = VirtualAddressInvalid;
    fn try_from(value: u64) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl From<PhysicalAddress> for VirtualAddress {
    fn from(value: PhysicalAddress) -> Self {
        Self::new(u64::from(value))
    }
}

impl From<u32> for VirtualAddress {
    fn from(value: u32) -> Self {
        Self::new(value as u64)
    }
}

impl From<u16> for VirtualAddress {
    fn from(value: u16) -> Self {
        Self::new(value as u64)
    }
}

impl From<u8> for VirtualAddress {
    fn from(value: u8) -> Self {
        Self::new(value as u64)
    }
}

impl TryFrom<*mut u8> for VirtualAddress {
    type Error = VirtualAddressInvalid;
    fn try_from(value: *mut u8) -> Result<Self, Self::Error> {
        Self::try_new(value as u64)
    }
}

impl TryFrom<*const u8> for VirtualAddress {
    type Error = VirtualAddressInvalid;
    fn try_from(value: *const u8) -> Result<Self, Self::Error> {
        Self::try_new(value as u64)
    }
}

impl Add<VirtualAddress> for VirtualAddress {
    type Output = Self;
    fn add(self, rhs: VirtualAddress) -> Self::Output {
        VirtualAddress::new(self.0 + rhs.0)
    }
}

impl Add<u64> for VirtualAddress {
    type Output = Self;
    fn add(self, rhs: u64) -> Self::Output {
        VirtualAddress::new(self.0 + rhs)
    }
}

impl Add<usize> for VirtualAddress {
    type Output = Self;
    fn add(self, rhs: usize) -> Self::Output {
        VirtualAddress::new(self.0 + rhs as u64)
    }
}

impl AddAssign<VirtualAddress> for VirtualAddress {
    fn add_assign(&mut self, rhs: VirtualAddress) {
        *self = *self + rhs.0;
    }
}

impl AddAssign<usize> for VirtualAddress {
    fn add_assign(&mut self, rhs: usize) {
        *self = *self + rhs as u64;
    }
}

impl AddAssign<u64> for VirtualAddress {
    fn add_assign(&mut self, rhs: u64) {
        *self = *self + rhs;
    }
}

impl Sub<VirtualAddress> for VirtualAddress {
    type Output = Self;
    fn sub(self, rhs: VirtualAddress) -> Self::Output {
        VirtualAddress::new(self.0 - rhs.0)
    }
}

impl Sub<u64> for VirtualAddress {
    type Output = Self;
    fn sub(self, rhs: u64) -> Self::Output {
        VirtualAddress::new(self.0 - rhs)
    }
}

impl Sub<usize> for VirtualAddress {
    type Output = Self;
    fn sub(self, rhs: usize) -> Self::Output {
        VirtualAddress::new(self.0 - rhs as u64)
    }
}

impl SubAssign<VirtualAddress> for VirtualAddress {
    fn sub_assign(&mut self, rhs: VirtualAddress) {
        *self = *self - rhs.0;
    }
}

impl SubAssign<usize> for VirtualAddress {
    fn sub_assign(&mut self, rhs: usize) {
        *self = *self - rhs as u64;
    }
}

impl SubAssign<u64> for VirtualAddress {
    fn sub_assign(&mut self, rhs: u64) {
        *self = *self - rhs;
    }
}

impl Debug for VirtualAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_tuple("VirtualAddress")
            .field(&format_args!("{:#X}", self.0))
            .finish()
    }
}

impl From<VirtualAddress> for usize {
    fn from(addr: VirtualAddress) -> Self {
        addr.0 as usize
    }
}

impl From<VirtualAddress> for *mut u8 {
    fn from(addr: VirtualAddress) -> Self {
        addr.0 as *mut u8
    }
}

impl From<VirtualAddress> for *const u8 {
    fn from(addr: VirtualAddress) -> Self {
        addr.0 as *const u8
    }
}
