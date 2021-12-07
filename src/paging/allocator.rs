use crate::arch::x86_64::registers::Cr3;
use crate::paging::page_table::{Level4, PageTable};
use crate::paging::phys_frame::PhysFrame;

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
