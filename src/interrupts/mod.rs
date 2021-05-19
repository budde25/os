use bit_field::BitField;
use lazy_static::lazy_static;

use crate::interrupts::gdt::Flags;
use handlers::ExceptionStackFrame;

mod gdt;
mod handlers;
mod idt;

pub trait Handler {}

pub type HandlerFunc = extern "x86-interrupt" fn(_: ExceptionStackFrame);
pub type HandlerFuncErrorCode = extern "x86-interrupt" fn(_: ExceptionStackFrame, _: u64);
pub type DivergingHandlerFuncErrorCode =
    extern "x86-interrupt" fn(_: ExceptionStackFrame, _: u64) -> !;
pub type DivergingHandlerFunc = extern "x86-interrupt" fn(_: ExceptionStackFrame) -> !;

impl Handler for HandlerFunc {}
impl Handler for HandlerFuncErrorCode {}
impl Handler for DivergingHandlerFunc {}
impl Handler for DivergingHandlerFuncErrorCode {}

#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum PrivilegeLevel {
    Ring0 = 0,
    Ring1 = 1,
    Ring2 = 2,
    Ring3 = 3,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct DescriptorTablePointer {
    limit: u16, // the inclusive limit from the base
    base: u64,  // Base addr
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct SegmentSelector(u16);

#[allow(dead_code)]
impl SegmentSelector {
    fn zero() -> Self {
        Self(0)
    }

    pub fn new(index: u16, level: PrivilegeLevel) -> Self {
        let mut selector = Self::zero();
        selector.set_priviledge_level(level);
        selector.set_index(index);
        selector
    }

    pub fn set_priviledge_level(&mut self, level: PrivilegeLevel) {
        self.0.set_bits(0..1, level as u16);
    }

    fn set_index(&mut self, index: u16) {
        self.0.set_bits(3..15, index);
    }

    fn code_segment() -> Self {
        let segment: u16;
        unsafe {
            asm!("mov {0:x}, cs", out(reg) segment, options(nostack, nomem, preserves_flags));
        };
        Self(segment)
    }
}

lazy_static! {
    /// 0, Divide By Zero
    /// 1, Debug
    /// 2, Non Maskable Interrupt
    /// 3, Breakpoint
    /// 4, Overflow
    /// 5, Bounds Range Exceeded
    /// 6, Invalid Opcode
    /// 7, Device Not Available
    /// 8, Double Fault
    /// 10, Invalid TSS
    /// 11, Segment Not Present
    /// 12, Stack Segment Fault
    /// 13, General Protection Fault
    /// 14, Page Fault
    static ref IDT: idt::InterruptDescriptorTable = {
        let mut idt = idt::InterruptDescriptorTable::new();
        idt.breakpoint.set_handler(handlers::breakpoint);
        idt.double_fault.set_handler(handlers::double_fault);
        idt
    };
}

lazy_static! {
    static ref GDT: gdt::GlobalDescriptorTable = {
        let mut gdt = gdt::GlobalDescriptorTable::new();
        // initialized to be empty and zero should be null anyway
        gdt.set_entry(1, gdt::Entry::new(0, 0x000FFFFF, gdt::Flags::code_ring_zero()));
        gdt.set_entry(2, gdt::Entry::new(0, 0x000FFFFF, gdt::Flags::data_ring_zero()));

        let mut code_ring_three = Flags::code_ring_zero();
        code_ring_three.set_priviledge_level(PrivilegeLevel::Ring3);
        let mut data_ring_three = Flags::data_ring_zero();
        data_ring_three.set_priviledge_level(PrivilegeLevel::Ring3);

        gdt.set_entry(3, gdt::Entry::new(0, 0x000FFFFF, code_ring_three));
        gdt.set_entry(4, gdt::Entry::new(0, 0x000FFFFF, data_ring_three));

        gdt
    };
}

pub fn init() {
    //    GDT.load();
    IDT.load();
}

#[cfg(test)]
mod tests {
    use crate::io::*;
    #[test_case]
    fn page_fault() {
        // trigger a page fault
        unsafe {
            //*(0xdeadbeef as *mut u64) = 42;
        };
    }

    /// Checks that we handle a breakpoint exeception by just returning
    #[test_case]
    fn breakpoint() {
        disable_uart();
        unsafe {
            asm!("int 3");
        }
        enable_uart();
    }
}
