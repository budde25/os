use crate::PhysicalAddress;

/// A 4kb physical memory frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub struct PhysFrame {
    address: PhysicalAddress,
}

impl PhysFrame {
    pub fn containing_address(address: PhysicalAddress) -> Self {
        Self {
            address: address.align_down(Self::size()),
        }
    }

    pub fn size() -> u64 {
        4096
    }

    pub fn address(&self) -> PhysicalAddress {
        self.address
    }
}
