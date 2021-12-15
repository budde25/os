pub mod allocator;
pub mod page_table;
pub mod phys_frame;

use allocator::Mapper;
use lazy_static::lazy_static;
use spin::Mutex;

use crate::kprintln;

lazy_static! {
    pub static ref MAPPER: Mutex<Mapper> = {
        let m = unsafe { Mapper::new() };
        Mutex::new(m)
    };
}

// Map all of physical memory to += phys mem offset
pub fn map_all_physical_memory() {
    // TODO this could use some serious refactoring
    use crate::address::phys::PhysicalAddress;
    use crate::address::sections::Section;
    use crate::address::SECTIONS;
    use page_table::{Level4, PageFlags, PageTable, PageTableEntry};

    const SIZE_2MIB: u64 = 0x200000;

    let mut m = MAPPER.lock();
    let p4 = m.p4_mut();

    let page_table_3 = SECTIONS[Section::PhysPageTable].start();

    let mut entry = PageTableEntry::new();
    let flags = PageFlags::PRESENT | PageFlags::WRITEABLE;
    entry.set_address(page_table_3, flags);
    p4[256] = entry;

    let p3 = p4.next_table_mut(256).unwrap();

    let mut page_table_2 = page_table_3 + core::mem::size_of::<PageTable<Level4>>();
    let mut addr_final = PhysicalAddress::new(0);
    for index in 0..32 {
        let mut entry = PageTableEntry::new();
        entry.set_address(page_table_2, flags);
        p3[index] = entry;
        page_table_2 += core::mem::size_of::<PageTable<Level4>>();
        let p2 = p3.next_table_mut(index).unwrap();

        for j in 0..512 {
            let flags_final = PageFlags::PRESENT | PageFlags::WRITEABLE | PageFlags::HUGE_PAGE;
            let mut entry = PageTableEntry::new();
            entry.set_address(addr_final, flags_final);
            p2[j] = entry;
            addr_final += SIZE_2MIB;
        }
    }
    kprintln!("All physical memory as been mapped");
}
