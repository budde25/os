use crate::paging::page_table::{Level4, PageTable};
use crate::paging::phys_frame::PhysFrame;
use crate::{arch::x86_64::registers::Cr3, VirtualAddress};

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
        None
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
    }
}
