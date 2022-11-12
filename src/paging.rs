use x86_64::PhysicalAddress;

use crate::consts::SIZE_1MIB;
use core::mem::size_of;
use spin::{Lazy, Mutex};
use x86_64::paging::allocator::Mapper;
use x86_64::paging::page_table::{Level4, PageFlags, PageTable};

pub static MAPPER: Lazy<Mutex<Mapper>> = Lazy::new(|| Mutex::new(unsafe { Mapper::new() }));

// Map all of physical memory to += phys mem offset
pub fn map_all_physical_memory(start_address: PhysicalAddress) {
    const SIZE_2MIB: u64 = SIZE_1MIB * 2;

    let mut m = MAPPER.lock();
    let p4 = m.p4_mut();

    let flags = PageFlags::PRESENT | PageFlags::WRITEABLE;
    p4[256].set_address(start_address, flags);

    let p3 = p4.next_table_mut(256).unwrap();
    let mut page_addr = PhysicalAddress::new(0);
    for p2_index in 0..32 {
        let p2_addr = PhysicalAddress::new(
            (start_address + (size_of::<PageTable<Level4>>() * p2_index)).into(),
        );
        p3[p2_index].set_address(p2_addr, flags);
        for page in p3.next_table_mut(p2_index).unwrap().iter_mut() {
            let page_flags = PageFlags::PRESENT | PageFlags::WRITEABLE | PageFlags::HUGE_PAGE;
            page.set_address(page_addr, page_flags);
            page_addr += SIZE_2MIB;
        }
    }
    crate::kprintln!("All physical memory as been mapped");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kprintln;

    // #[test_case]
    pub fn debug_print_p4_table() {
        let m = MAPPER.lock();
        let p4 = m.p4();

        for (i, entry) in p4.iter().enumerate() {
            if !entry.is_unused() {
                kprintln!("L4 Entry {}: {:#?}", i, entry);
            }
        }
    }

    // #[test_case]
    pub fn debug_print_p3_table() {
        let m = MAPPER.lock();
        let p4 = m.p4();
        let p3 = p4.next_table(0).unwrap();

        for (i, entry) in p3.iter().enumerate() {
            if !entry.is_unused() {
                kprintln!("L3 Entry {}: {:#?}", i, entry);
            }
        }
    }

    //#[test_case]
    pub fn debug_print_p2_table() {
        let m = MAPPER.lock();
        let p4 = m.p4();
        let p3 = p4.next_table(0).unwrap();
        let p2 = p3.next_table(0).unwrap();

        for (i, entry) in p2.iter().enumerate() {
            if !entry.is_unused() {
                kprintln!("L2 Entry {}: {:#?}", i, entry);
            }
        }
    }
}
