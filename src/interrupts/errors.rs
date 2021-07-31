use crate::address::virt::VirtualAddress;
use crate::interrupts::{rflags::RFlags, SegmentSelector};
use bit_field::BitField;
use bitflags::bitflags;
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

// grabbed from x86_64 crate
bitflags! {
    /// Describes an page fault error code.
    #[repr(transparent)]
    pub struct PageFaultErrorCode: u64 {
        /// If this flag is set, the page fault was caused by a page-protection violation,
        /// else the page fault was caused by a not-present page.
        const PROTECTION_VIOLATION = 1;

        /// If this flag is set, the memory access that caused the page fault was a write.
        /// Else the access that caused the page fault is a memory read. This bit does not
        /// necessarily indicate the cause of the page fault was a read or write violation.
        const CAUSED_BY_WRITE = 1 << 1;

        /// If this flag is set, an access in user mode (CPL=3) caused the page fault. Else
        /// an access in supervisor mode (CPL=0, 1, or 2) caused the page fault. This bit
        /// does not necessarily indicate the cause of the page fault was a privilege violation.
        const USER_MODE = 1 << 2;

        /// If this flag is set, the page fault is a result of the processor reading a 1 from
        /// a reserved field within a page-translation-table entry.
        const MALFORMED_TABLE = 1 << 3;

        /// If this flag is set, it indicates that the access that caused the page fault was an
        /// instruction fetch.
        const INSTRUCTION_FETCH = 1 << 4;
    }
}
