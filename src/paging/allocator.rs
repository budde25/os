use super::tlb;
use crate::consts::{KHEAP_START, SIZE_1KIB};
use crate::paging::page_table::{Level4, PageTable};
use crate::paging::phys_frame::PhysFrame;
use crate::{kdbg, PhysicalAddress};
use crate::{registers::control::Cr3, VirtualAddress};
use core::sync::atomic::{AtomicU64, Ordering};

use super::page_table::PageFlags;

pub struct FreeList {
    list: [PhysFrame; 256],
}

pub trait Allocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame>;
    fn dealloc_frame(&mut self, frame: PhysFrame);
}

pub struct Mapper {
    p4: &'static mut PageTable<Level4>,
}

impl Mapper {
    pub unsafe fn new() -> Self {
        let mut frame = Cr3::read().frame();
        Self::from_p4_unchecked(&mut frame)
    }

    unsafe fn from_p4_unchecked(frame: &mut PhysFrame) -> Self {
        let virt = frame.address();

        Self {
            p4: &mut *(u64::from(virt) as *mut PageTable<Level4>),
        }
    }

    pub fn p4(&self) -> &PageTable<Level4> {
        &*self.p4
    }

    pub fn p4_mut(&mut self) -> &mut PageTable<Level4> {
        &mut *self.p4
    }
}

impl Allocator for Mapper {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        // TODO: support larger allocations
        let p3 = self.p4_mut().next_table_mut(0).expect("Invalid frame");
        let p2 = p3.next_table_mut(0).expect("Invalid frame");
        for item in p2.iter_mut() {
            if item.is_unused() {
                // TODO: this will not allow a dealloc to actually reclaim any memory
                static ADDR: AtomicU64 = AtomicU64::new(KHEAP_START);
                // found one now allocate
                item.set_address(
                    PhysicalAddress::new(ADDR.fetch_add(SIZE_1KIB, Ordering::Relaxed)),
                    PageFlags::PRESENT | PageFlags::WRITEABLE,
                );
                kdbg!(&item);
                return item.frame();
            }
        }
        // for now lets panic to be aware of our usage
        // later we will return none
        panic!("PageTable out of memory!");
    }

    fn dealloc_frame(&mut self, frame: PhysFrame) {
        let virt: VirtualAddress = frame.address().into();
        let p3 = self
            .p4_mut()
            .next_table_mut(virt.p4_index().into())
            .expect("Invalid frame");
        let p2 = p3
            .next_table_mut(virt.p3_index().into())
            .expect("Invalid frame");
        p2[virt.p2_index()].set_unused();
        tlb::flush(virt);
    }
}
