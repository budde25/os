use bit_field::BitField;
use core::{fmt, usize};
use lazy_static::lazy_static;

use gdt::GlobalDescriptorTable;

use tss::TaskStateSegment;

pub mod errors;
pub mod gdt;
pub mod idt;
pub mod rflags;
pub mod tss;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

pub struct Selectors {
    pub kernel_code_segment: SegmentSelector,
    pub kernel_data_segment: SegmentSelector,
    pub tss_segment: SegmentSelector,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum PrivilegeLevel {
    Ring0 = 0,
    Ring1 = 1,
    Ring2 = 2,
    Ring3 = 3,
}

impl From<u16> for PrivilegeLevel {
    fn from(num: u16) -> Self {
        match num {
            0 => PrivilegeLevel::Ring0,
            1 => PrivilegeLevel::Ring1,
            2 => PrivilegeLevel::Ring2,
            3 => PrivilegeLevel::Ring3,
            i => panic!("{} is not a valid privilege level", i),
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct DescriptorTablePointer {
    limit: u16, // the inclusive limit from the base
    base: u64,  // Base addr
}

#[derive(Clone, Copy, Default)]
#[repr(transparent)]
pub struct SegmentSelector(u16);

impl SegmentSelector {
    pub fn new(index: u16, level: PrivilegeLevel) -> Self {
        SegmentSelector(index << 3 | (level as u16))
    }

    pub fn set_privilege_level(&mut self, level: PrivilegeLevel) {
        self.0.set_bits(0..2, level as u16);
    }

    pub fn get_privilege_level(&self) -> PrivilegeLevel {
        self.0.get_bits(0..2).into()
    }

    pub fn get_index(&self) -> u16 {
        self.0 >> 3
    }

    // gets the code segment index
    fn code_segment() -> Self {
        let segment: u16;
        unsafe {
            asm!("mov {0:x}, cs", out(reg) segment, options(nostack, nomem, preserves_flags));
        };
        Self(segment)
    }

    // gets the data segment index
    fn data_segment() -> Self {
        let segment: u16;
        unsafe {
            asm!("mov {0:x}, ds", out(reg) segment, options(nostack, nomem, preserves_flags));
        };
        Self(segment)
    }
}

impl fmt::Debug for SegmentSelector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = f.debug_struct("Segment Selector");
        s.field("privilege level", &self.get_privilege_level());
        s.field("index", &self.get_index());
        s.finish()
    }
}

lazy_static! {
    pub static ref IDT: idt::InterruptDescriptorTable = {
        use idt::handlers::*;
        use idt::InterruptIndex;

        let mut idt = idt::InterruptDescriptorTable::new();
        idt.divide_by_zero.set_handler(divide_by_zero);
        idt.debug.set_handler(debug);
        idt.non_maskable_interrupt.set_handler(non_maskable_interrupt);
        idt.breakpoint.set_handler(breakpoint);
        idt.overflow.set_handler(overflow);
        idt.bound_range_exceeded.set_handler(bound_range_exceeded);
        idt.invalid_opcode.set_handler(invalid_opcode);
        idt.device_not_available.set_handler(device_not_available);

        // double fault handler
        idt.double_fault.set_handler(double_fault);
        idt.double_fault.options.set_stack_index(DOUBLE_FAULT_IST_INDEX);

        idt.invalid_tss.set_handler(invalid_tss);
        idt.segment_not_present.set_handler(segment_not_present);
        idt.stack_segment_fault.set_handler(stack_segment_fault);
        idt.general_protection_fault.set_handler(general_protection_fault);
        idt.page_fault.set_handler(page_fault);
        idt.x87_floating_point.set_handler(x87_floating_point);
        idt.alignment_check.set_handler(alignment_check);
        idt.machine_check.set_handler(machine_check);
        idt.simd_floating_point.set_handler(simd_floating_point);
        idt.virtualization.set_handler(virtualization);
        idt.security_exception.set_handler(security_exception);

        // interrupt handlers
        idt.interrupts[InterruptIndex::Timer as usize].set_handler(timer);
        idt.interrupts[InterruptIndex::Keyboard as usize].set_handler(keyboard);

        idt
    };

    pub static ref GDT: (gdt::GlobalDescriptorTable, Selectors) = {
        use gdt::{Entry, Flags};

        let mut gdt = GlobalDescriptorTable::new();

        // initialized to be empty and zero should be null anyway
        let kernel_code_segment = gdt.push(Entry::new(0, Flags::CODE_PL_ZERO));
        let kernel_data_segment = gdt.push(Entry::new(0, Flags::DATA_PL_ZERO));
        gdt.push(Entry::new(0, Flags::CODE_PL_THREE));
        gdt.push(Entry::new(0, Flags::DATA_PL_THREE));

        // tss
        let (tss_segment_1, tss_segment_2) = Entry::tss(&TSS);
        let tss_segment = gdt.push(tss_segment_1);
        gdt.push(tss_segment_2);

        (gdt, Selectors {kernel_code_segment, kernel_data_segment, tss_segment})
    };

    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::zero();

        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = unsafe { &STACK as *const _ as u64};
            stack_start + STACK_SIZE as u64
        };
        tss
    };

}

pub fn disable_interrupts() {
    unsafe { asm!("cli", options(nomem, nostack)) };
}

pub fn enable_interrupts() {
    unsafe { asm!("sti", options(nomem, nostack)) };
}

pub fn halt() {
    unsafe { asm!("hlt", options(nomem, nostack, preserves_flags)) };
}

pub fn interrupts_enabled() -> bool {
    use rflags::RFlags;
    RFlags::read().contains(RFlags::INTERRUPT_ENABLE)
}

pub fn halt_loop() -> ! {
    loop {
        halt();
    }
}

pub fn init() {
    GDT.0.load();
    IDT.load();

    unsafe {
        gdt::load_cs(GDT.1.kernel_code_segment);
        // gdt::load_ds(GDT.1.kernel_data_segment);
        gdt::load_tss(GDT.1.tss_segment);
    }
}

// Run a chunk of code without interrupts enabled
pub fn without_interrupts<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let int_enabled = interrupts_enabled();
    if int_enabled {
        disable_interrupts();
    }

    let ret = f();

    if int_enabled {
        enable_interrupts();
    }

    ret
}

#[cfg(test)]
mod tests {
    use crate::io::*;

    #[test_case]
    fn page_fault() {
        uart_disable();
        //*(0xdeadbeef as *mut u64) = 42;
        uart_enable();
    }

    /// Checks that we handle a breakpoint exception by just returning
    #[test_case]
    fn breakpoint() {
        uart_disable();
        unsafe {
            asm!("int 3");
        }
        uart_enable();
    }
}
