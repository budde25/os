use crate::address::phys::PhysicalAddress;
use core::fmt::{self, Debug};

pub struct SectionRange {
    start: PhysicalAddress, // inclusive
    end: PhysicalAddress,   // exclusive
}

impl SectionRange {
    fn new(start: u64, end: u64) -> Self {
        Self {
            start: PhysicalAddress::new(start),
            end: PhysicalAddress::new(end),
        }
    }
}

impl Debug for SectionRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct Hex(u64);
        impl Debug for Hex {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{:X}", self.0)
            }
        }

        f.debug_struct("SectionRange")
            .field("start", &Hex(u64::from(self.start)))
            .field("end", &Hex(u64::from(self.end)))
            .finish()
    }
}

#[derive(Debug)]
pub enum Section {
    TEXT,
    RODATA,
    DATA,
    BSS,
    UNKNOWN,
}

impl Section {
    pub fn containing_adrress(pa: &PhysicalAddress) -> Self {
        extern "C" {
            static __text_start: usize;
            static __text_end: usize;
            static __rodata_start: usize;
            static __rodata_end: usize;
            static __data_start: usize;
            static __data_end: usize;
            static __bss_start: usize;
            static __bss_end: usize;
        }

        let text_start = unsafe { &__text_start as *const _ as u64 };
        let text_end = unsafe { &__text_end as *const _ as u64 };
        let rodata_start = unsafe { &__rodata_start as *const _ as u64 };
        let rodata_end = unsafe { &__rodata_end as *const _ as u64 };
        let data_start = unsafe { &__data_start as *const _ as u64 };
        let data_end = unsafe { &__data_end as *const _ as u64 };
        let bss_start = unsafe { &__bss_start as *const _ as u64 };
        let bss_end = unsafe { &__bss_end as *const _ as u64 };

        let pau = u64::from(*pa);
        if pau >= text_start && pau < text_end {
            Section::TEXT
        } else if pau >= rodata_start && pau < rodata_end {
            Section::RODATA
        } else if pau >= data_start && pau < data_end {
            Section::DATA
        } else if pau >= bss_start && pau < bss_end {
            Section::BSS
        } else {
            Section::UNKNOWN
        }
    }
}
