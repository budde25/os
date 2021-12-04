use bit_field::BitField;
use core::convert::TryFrom;
use core::fmt::{self, Debug, Formatter};
use core::ops::{Add, AddAssign, Sub, SubAssign};

use super::{align_down, align_up};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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

    pub fn is_null(self) -> bool {
        self.0 == 0
    }

    pub fn align_down<U>(self, align: U) -> Self
    where
        U: Into<u64>,
    {
        Self(align_down(self.0, align.into()))
    }

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
        f.debug_tuple("PhysicalAddress")
            .field(&format_args!("{:#x}", self.0))
            .finish()
    }
}
