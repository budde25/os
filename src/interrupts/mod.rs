use bit_field::BitField;
use lazy_static::lazy_static;

use gdt::GlobalDescriptorTable;
use handlers::ExceptionStackFrame;

use tss::TaskStateSegment;

mod gdt;
mod handlers;
mod idt;
mod tss;

pub trait Handler {}

pub type HandlerFunc = extern "x86-interrupt" fn(_: ExceptionStackFrame);
pub type HandlerFuncErrorCode = extern "x86-interrupt" fn(_: ExceptionStackFrame, _: u64);
pub type DivergingHandlerFuncErrorCode =
    extern "x86-interrupt" fn(_: ExceptionStackFrame, _: u64) -> !;
pub type DivergingHandlerFunc = extern "x86-interrupt" fn(_: ExceptionStackFrame) -> !;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

impl Handler for HandlerFunc {}
impl Handler for HandlerFuncErrorCode {}
impl Handler for DivergingHandlerFunc {}
impl Handler for DivergingHandlerFuncErrorCode {}

struct Selectors {
    kernel_code_segment: SegmentSelector,
    tss_segment: SegmentSelector,
}

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
    static ref IDT: idt::InterruptDescriptorTable = {
        let mut idt = idt::InterruptDescriptorTable::new();
        idt.divide_by_zero.set_handler(handlers::divide_by_zero);
        idt.debug.set_handler(handlers::debug);
        idt.non_maskable_interrupt.set_handler(handlers::non_maskable_interrupt);
        idt.breakpoint.set_handler(handlers::breakpoint);
        idt.overflow.set_handler(handlers::overflow);
        idt.bound_range_exceeded.set_handler(handlers::bound_range_exceeded);
        idt.invalid_opcode.set_handler(handlers::invalid_opcode);
        idt.device_not_available.set_handler(handlers::device_not_available);
        idt.double_fault.set_handler(handlers::double_fault);
        idt.invalid_tss.set_handler(handlers::invalid_tss);
        idt.segment_not_present.set_handler(handlers::segment_not_present);
        idt.stack_segment_fault.set_handler(handlers::stack_segment_fault);
        idt.general_protection_fault.set_handler(handlers::general_protection_fault);
        idt.page_fault.set_handler(handlers::page_fault);
        idt.x87_floating_point.set_handler(handlers::x87_floating_point);
        idt.alignment_check.set_handler(handlers::alignment_check);
        idt.machine_check.set_handler(handlers::machine_check);
        idt.simd_floating_point.set_handler(handlers::simd_floating_point);
        idt.virtualization.set_handler(handlers::virtualization);
        idt.security_exception.set_handler(handlers::security_exception);
        idt
    };

    static ref GDT: (gdt::GlobalDescriptorTable, Selectors) = {
        use gdt::{Entry, Flags};

        let mut gdt = GlobalDescriptorTable::new();

        // initialized to be empty and zero should be null anyway
        let kernel_code_segment = gdt.set_entry(1, Entry::new(0, Flags::CODE_PL_ZERO));
        gdt.set_entry(2, Entry::new(0, Flags::DATA_PL_ZERO));
        gdt.set_entry(3, Entry::new(0, Flags::CODE_PL_THREE));
        gdt.set_entry(4, Entry::new(0, Flags::DATA_PL_THREE));

        // tss
        let (tss_segment_1, tss_segment_2) = Entry::tss(&TSS);
        let tss_segment = gdt.set_entry(5, tss_segment_1);
        gdt.set_entry(6, tss_segment_2);

        (gdt, Selectors {kernel_code_segment, tss_segment})
    };

    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::zero();

        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = unsafe { &STACK as *const _ as u64};
            let stack_end = stack_start + STACK_SIZE as u64;
            stack_end
        };
        tss
    };

}

pub fn init() {
    GDT.0.load();
    IDT.load();

    unsafe {
        //asm!("cli", options(nomem, nostack));
        gdt::load_cs(GDT.1.kernel_code_segment);
        gdt::load_tss(GDT.1.tss_segment);
        //asm!("sti", options(nomem, nostack));
    }
}

#[cfg(test)]
mod tests {
    use crate::{interrupts::gdt::Entry, io::*};

    #[test_case]
    fn page_fault() {
        disable_uart();
        unsafe {
            //        *(0xdeadbeef as *mut u64) = 42;
        };
        enable_uart();
    }

    /// Checks that we handle a breakpoint exeception by just returning
    #[test_case]
    fn breakpoint() {
        disable_uart();
        unsafe {
            //    asm!("int 3");
        }
        enable_uart();
    }
}
