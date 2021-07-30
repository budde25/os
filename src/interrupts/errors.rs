use crate::address::virt::VirtualAddress;
use crate::interrupts::{rflags::RFlags, SegmentSelector};
use bit_field::BitField;
use core::fmt::{self, Debug};

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct ExceptionStackFrame {
    pub instruction_pointer: VirtualAddress,
    pub code_segment: SegmentSelector,
    _reserved_1: [u8; 6],
    cpu_flags: RFlags,
    stack_pointer: VirtualAddress,
    stack_segment: SegmentSelector,
    _reserved_2: [u8; 6],
}

impl Debug for ExceptionStackFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let instruction_pointer = self.instruction_pointer;
        let code_segment = self.code_segment;
        let cpu_flags = self.cpu_flags;
        let stack_pointer = self.stack_pointer;
        let stack_segment = self.stack_segment;

        let mut s = f.debug_struct("ExceptionStackFrame");
        s.field("instruction_pointer", &instruction_pointer);
        s.field("code_segment", &code_segment);
        s.field("cpu_flags", &cpu_flags);
        s.field("stack_pointer", &stack_pointer);
        s.field("stack_segment", &stack_segment);
        s.finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DescriptorTable {
    Gdt,
    Idt,
    Ldt,
}

// needs to be transparent u64, otherwise on LLVM 12 we get "unsupported x86 interrupt prototype"
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct SelectorError {
    flags: u64,
}

impl SelectorError {
    fn external(&self) -> bool {
        self.flags.get_bit(0)
    }

    fn descriptor_table(&self) -> DescriptorTable {
        match self.flags.get_bits(1..3) {
            0 => DescriptorTable::Gdt,
            1 => DescriptorTable::Idt,
            2 => DescriptorTable::Ldt,
            3 => DescriptorTable::Idt,
            _ => unreachable!(),
        }
    }

    fn selector_index(&self) -> u64 {
        self.flags.get_bits(3..16)
    }
}

impl Debug for SelectorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = f.debug_struct("SelectorError");
        s.field("External", &self.external());
        s.field("DescriptorTable", &self.descriptor_table());
        s.field("SelectorIndex", &self.selector_index());
        s.finish()
    }
}
