use crate::address::phys::PhysicalAddress;
use bit_field::BitField;
use bitflags::bitflags;
use core::{
    convert::TryInto,
    fmt::{self, Debug},
};

pub struct Page(u64);

impl Page {
    fn get_physical_addr(&self) -> PhysicalAddress {
        self.0.get_bits(12..52).try_into().unwrap()
    }
}

impl Debug for Page {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let flags = PageFlags::from_bits_truncate(self.0);
        let addr = self.get_physical_addr();
        f.debug_struct("Page")
            .field("Address", &addr)
            .field("Flags", &flags)
            .finish()
    }
}

bitflags! {
    pub struct PageFlags: u64 {
        const PRESENT = 1;
        const WRITEABLE = 1 << 1;
        const USER_ACCESSIBLE = 1 << 2;
        const WRITE_THROUGH_CACHING = 1 << 3;
        const DISABLE_CACHE = 1 << 4;
        const ACCESSED = 1 << 5; // set by cpu
        const DIRTY = 1 << 6; //set by cpu
        const HUGE_PAGE = 1 << 7; // must be zero on pe, pdpte
        const GLOBAL = 1 << 8;
        const AVAILABLE_1 = 1 << 9;
        const AVAILABLE_2 = 1 << 10;
        const AVAILABLE_3 = 1 << 11;
        const AVAILABLE_4 = 1 << 52;
        const AVAILABLE_5 = 1 << 53;
        const AVAILABLE_6 = 1 << 54;
        const AVAILABLE_7 = 1 << 55;
        const AVAILABLE_8 = 1 << 56;
        const AVAILABLE_9 = 1 << 57;
        const AVAILABLE_10 = 1 << 58;
        const AVAILABLE_11 = 1 << 59;
        const AVAILABLE_12 = 1 << 60;
        const AVAILABLE_13 = 1 << 61;
        const AVAILABLE_14 = 1 << 62;
        const NO_EXECUTE = 1 << 63;
    }
}
