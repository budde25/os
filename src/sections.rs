use spin::lazy::Lazy;
use x86_64::PhysicalAddress;

use core::{
    fmt::{self, Debug},
    ops::Index,
};

pub static SECTIONS: Lazy<Sections> = Lazy::new(Sections::new);

#[derive(Debug)]
pub struct Sections {
    text: SectionRange,
    rodata: SectionRange,
    data: SectionRange,
    bss: SectionRange,
    page_table: SectionRange,
    phys_page_table: SectionRange,
}

impl Sections {
    pub fn new() -> Self {
        extern "C" {
            static __text_start: usize;
            static __text_end: usize;
            static __rodata_start: usize;
            static __rodata_end: usize;
            static __data_start: usize;
            static __data_end: usize;
            static __bss_start: usize;
            static __bss_end: usize;
            static __page_table_start: usize;
            static __page_table_end: usize;
            static __page_table_2_start: usize;
            static __page_table_2_end: usize;
        }

        let text_start = unsafe { &__text_start as *const _ as u64 };
        let text_end = unsafe { &__text_end as *const _ as u64 };
        let rodata_start = unsafe { &__rodata_start as *const _ as u64 };
        let rodata_end = unsafe { &__rodata_end as *const _ as u64 };
        let data_start = unsafe { &__data_start as *const _ as u64 };
        let data_end = unsafe { &__data_end as *const _ as u64 };
        let bss_start = unsafe { &__bss_start as *const _ as u64 };
        let bss_end = unsafe { &__bss_end as *const _ as u64 };
        let page_table_start = unsafe { &__page_table_start as *const _ as u64 };
        let page_table_end = unsafe { &__page_table_end as *const _ as u64 };
        let page_table_2_start = unsafe { &__page_table_2_start as *const _ as u64 };
        let page_table_2_end = unsafe { &__page_table_2_end as *const _ as u64 };

        Self {
            text: SectionRange::new(text_start, text_end),
            rodata: SectionRange::new(rodata_start, rodata_end),
            data: SectionRange::new(data_start, data_end),
            bss: SectionRange::new(bss_start, bss_end),
            page_table: SectionRange::new(page_table_start, page_table_end),
            phys_page_table: SectionRange::new(page_table_2_start, page_table_2_end),
        }
    }

    pub fn containing_address(&self, pa: &PhysicalAddress) -> Section {
        if pa >= &self.text.start && pa < &self.text.end {
            Section::Text
        } else if pa >= &self.rodata.start && pa < &self.rodata.end {
            Section::RoData
        } else if pa >= &self.data.start && pa < &self.data.end {
            Section::Data
        } else if pa >= &self.bss.start && pa < &self.bss.end {
            Section::Bss
        } else if pa >= &self.page_table.start && pa < &self.page_table.end {
            Section::PageTable
        } else if pa >= &self.phys_page_table.start && pa < &self.phys_page_table.end {
            Section::PhysPageTable
        } else {
            Section::Unknown
        }
    }
}

impl Default for Sections {
    fn default() -> Self {
        Self::new()
    }
}

impl Index<Section> for Sections {
    type Output = SectionRange;
    fn index(&self, index: Section) -> &Self::Output {
        match index {
            Section::Text => &self.text,
            Section::RoData => &self.rodata,
            Section::Data => &self.data,
            Section::Bss => &self.bss,
            Section::PageTable => &self.page_table,
            Section::PhysPageTable => &self.phys_page_table,
            Section::Unknown => panic!("Unkown index"),
        }
    }
}

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

    // Start of the section inclusive
    pub fn start(&self) -> PhysicalAddress {
        self.start
    }

    // End of the section exclusive
    pub fn end(&self) -> PhysicalAddress {
        self.end
    }
}

impl Debug for SectionRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct Hex(u64);
        impl Debug for Hex {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{:#X}", self.0)
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
    Text,
    RoData,
    Data,
    Bss,
    PageTable,
    PhysPageTable,
    Unknown,
}
