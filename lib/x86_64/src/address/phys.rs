use super::virt::VirtualAddress;
use bit_field::BitField;
use core::convert::TryFrom;
use core::fmt::{self, Debug, Formatter};
use core::ops::{Add, AddAssign, Sub, SubAssign};
use derive_more::{Binary, Display, LowerHex, UpperHex};

use super::{align_down, align_up};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Display, UpperHex, LowerHex, Binary)]
#[display(fmt = "P_0x{:X}", _0)]
#[repr(transparent)]

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
            _ => Err(PhysicalAddressInvalid(address)),
        }
    }

    pub const fn truncate_new(address: u64) -> Self {
        Self(address % (1 << 52))
    }

    pub const fn as_ptr<T>(&self) -> *const T {
        use crate::KERNEL_OFFSET;
        (self.0 + KERNEL_OFFSET) as *const T
    }

    pub const fn as_mut_ptr<T>(&self) -> *mut T {
        self.as_ptr::<T>() as *mut T
    }

    pub const fn is_null(self) -> bool {
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
}

impl TryFrom<u64> for PhysicalAddress {
    type Error = PhysicalAddressInvalid;
    fn try_from(value: u64) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl TryFrom<usize> for PhysicalAddress {
    type Error = PhysicalAddressInvalid;
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Self::try_new(value as u64)
    }
}

impl From<VirtualAddress> for PhysicalAddress {
    fn from(value: VirtualAddress) -> Self {
        Self::new(u64::from(value))
    }
}

impl From<u32> for PhysicalAddress {
    fn from(value: u32) -> Self {
        Self::new(value as u64)
    }
}

impl From<u16> for PhysicalAddress {
    fn from(value: u16) -> Self {
        Self::new(value as u64)
    }
}

impl From<u8> for PhysicalAddress {
    fn from(value: u8) -> Self {
        Self::new(value as u64)
    }
}

impl Add<PhysicalAddress> for PhysicalAddress {
    type Output = Self;
    fn add(self, rhs: PhysicalAddress) -> Self::Output {
        PhysicalAddress::new(self.0 + rhs.0)
    }
}

impl Add<u64> for PhysicalAddress {
    type Output = Self;
    fn add(self, rhs: u64) -> Self::Output {
        PhysicalAddress::new(self.0 + rhs)
    }
}

impl Add<usize> for PhysicalAddress {
    type Output = Self;
    fn add(self, rhs: usize) -> Self::Output {
        PhysicalAddress::new(self.0 + rhs as u64)
    }
}

impl AddAssign<PhysicalAddress> for PhysicalAddress {
    fn add_assign(&mut self, rhs: PhysicalAddress) {
        *self = *self + rhs.0;
    }
}

impl AddAssign<usize> for PhysicalAddress {
    fn add_assign(&mut self, rhs: usize) {
        *self = *self + rhs as u64;
    }
}

impl AddAssign<u64> for PhysicalAddress {
    fn add_assign(&mut self, rhs: u64) {
        *self = *self + rhs;
    }
}

impl Sub<PhysicalAddress> for PhysicalAddress {
    type Output = Self;
    fn sub(self, rhs: PhysicalAddress) -> Self::Output {
        PhysicalAddress::new(self.0 - rhs.0)
    }
}

impl Sub<u64> for PhysicalAddress {
    type Output = Self;
    fn sub(self, rhs: u64) -> Self::Output {
        PhysicalAddress::new(self.0 - rhs)
    }
}

impl Sub<usize> for PhysicalAddress {
    type Output = Self;
    fn sub(self, rhs: usize) -> Self::Output {
        PhysicalAddress::new(self.0 - rhs as u64)
    }
}

impl SubAssign<PhysicalAddress> for PhysicalAddress {
    fn sub_assign(&mut self, rhs: PhysicalAddress) {
        *self = *self - rhs.0;
    }
}

impl SubAssign<usize> for PhysicalAddress {
    fn sub_assign(&mut self, rhs: usize) {
        *self = *self - rhs as u64;
    }
}

impl SubAssign<u64> for PhysicalAddress {
    fn sub_assign(&mut self, rhs: u64) {
        *self = *self - rhs;
    }
}

impl Debug for PhysicalAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("PhysicalAddress")
            .field("address", &format_args!("{:#X}", self.0))
            .finish()
    }
}

impl From<PhysicalAddress> for u64 {
    fn from(addr: PhysicalAddress) -> Self {
        addr.0
    }
}

impl From<PhysicalAddress> for usize {
    fn from(addr: PhysicalAddress) -> Self {
        addr.0 as usize
    }
}
