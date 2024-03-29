use super::{DescriptorTablePointer, SegmentSelector};
use crate::interrupts::errors::{ExceptionStackFrame, PageFaultErrorCode, SelectorError};
use bit_field::BitField;
use bitflags::bitflags;
use core::arch::asm;
use core::fmt::{self, Debug, Formatter};
use core::marker::PhantomData;
use core::usize;
use derive_more::From;

pub trait Handler {}

pub type HandlerFunc = extern "x86-interrupt" fn(_: ExceptionStackFrame);
pub type HandlerFuncErrorCode = extern "x86-interrupt" fn(_: ExceptionStackFrame, _: u64);
pub type SegmentNotPresentHandlerFunc =
    extern "x86-interrupt" fn(_: ExceptionStackFrame, _: SelectorError);
pub type PageFaultHandlerFunc =
    extern "x86-interrupt" fn(_: ExceptionStackFrame, _: PageFaultErrorCode);
pub type DivergingHandlerFuncErrorCode =
    extern "x86-interrupt" fn(_: ExceptionStackFrame, _: u64) -> !;
pub type DivergingHandlerFunc = extern "x86-interrupt" fn(_: ExceptionStackFrame) -> !;

impl Handler for HandlerFunc {}
impl Handler for HandlerFuncErrorCode {}
impl Handler for DivergingHandlerFunc {}
impl Handler for DivergingHandlerFuncErrorCode {}

/// Interrupt Descriptor Table
#[derive(Debug)]
#[repr(C)]
#[repr(align(16))]
pub struct InterruptDescriptorTable {
    pub divide_by_zero: Entry<HandlerFunc>,
    pub debug: Entry<HandlerFunc>,
    pub non_maskable_interrupt: Entry<HandlerFunc>,
    pub breakpoint: Entry<HandlerFunc>,
    pub overflow: Entry<HandlerFunc>,
    pub bound_range_exceeded: Entry<HandlerFunc>,
    pub invalid_opcode: Entry<HandlerFunc>,
    pub device_not_available: Entry<HandlerFunc>,
    pub double_fault: Entry<DivergingHandlerFuncErrorCode>,
    coprocessor_segment_overrun: Entry<HandlerFunc>,
    pub invalid_tss: Entry<HandlerFuncErrorCode>,
    pub segment_not_present: Entry<SegmentNotPresentHandlerFunc>,
    pub stack_segment_fault: Entry<HandlerFuncErrorCode>,
    pub general_protection_fault: Entry<HandlerFuncErrorCode>,
    pub page_fault: Entry<PageFaultHandlerFunc>,
    reserved_1: Entry<HandlerFunc>,
    pub x87_floating_point: Entry<HandlerFunc>,
    pub alignment_check: Entry<HandlerFuncErrorCode>,
    pub machine_check: Entry<DivergingHandlerFunc>,
    pub simd_floating_point: Entry<HandlerFunc>,
    pub virtualization: Entry<HandlerFunc>,
    reserved_2: [Entry<HandlerFunc>; 9],
    pub security_exception: Entry<HandlerFuncErrorCode>,
    reserved_3: Entry<HandlerFunc>,
    pub interrupts: [Entry<HandlerFunc>; 256 - 32],
}

impl InterruptDescriptorTable {
    pub fn new() -> Self {
        Self {
            divide_by_zero: Entry::empty(),
            debug: Entry::empty(),
            non_maskable_interrupt: Entry::empty(),
            breakpoint: Entry::empty(),
            overflow: Entry::empty(),
            bound_range_exceeded: Entry::empty(),
            invalid_opcode: Entry::empty(),
            device_not_available: Entry::empty(),
            double_fault: Entry::empty(),
            coprocessor_segment_overrun: Entry::empty(),
            invalid_tss: Entry::empty(),
            segment_not_present: Entry::empty(),
            stack_segment_fault: Entry::empty(),
            general_protection_fault: Entry::empty(),
            page_fault: Entry::empty(),
            reserved_1: Entry::empty(),
            x87_floating_point: Entry::empty(),
            alignment_check: Entry::empty(),
            machine_check: Entry::empty(),
            simd_floating_point: Entry::empty(),
            virtualization: Entry::empty(),
            reserved_2: [Entry::empty(); 9],
            security_exception: Entry::empty(),
            reserved_3: Entry::empty(),
            interrupts: [Entry::empty(); 256 - 32],
        }
    }

    pub fn load(&'static self) {
        use core::mem::size_of;
        let ptr = DescriptorTablePointer {
            base: self as *const _ as u64,
            limit: (size_of::<Self>() - 1) as u16,
        };
        unsafe {
            asm!("lidt [{}]", in(reg) &ptr, options(readonly, nostack, preserves_flags));
        }
    }
}

impl Default for InterruptDescriptorTable {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct Entry<T> {
    handler_low: u16,          // offset bits 0..15
    selector: SegmentSelector, // a code segment selector in GDT or LDT
    pub options: Options,      // type and attributes
    handler_middle: u16,       // offset bits 16..31
    handler_high: u32,         // offset bits 32..63
    zero: u32,                 // reserved
    phantom: PhantomData<T>,
}

impl<T> Entry<T> {
    /// Set a present entry with handler
    fn set_handler_addr(&mut self, handler: u64) {
        self.handler_low = handler as u16;
        self.handler_middle = (handler >> 16) as u16;
        self.handler_high = (handler >> 32) as u32;
        self.options = Options::default();
        self.selector = SegmentSelector::code_segment();
    }

    /// Create a non present entry
    fn empty() -> Self {
        Self {
            selector: SegmentSelector::default(),
            handler_low: 0,
            handler_middle: 0,
            handler_high: 0,
            options: Options::zero(),
            zero: 0,
            phantom: PhantomData,
        }
    }

    fn get_handler(&self) -> u64 {
        let mut handler: u64 = self.handler_low as u64;
        handler |= (self.handler_middle as u64) << 16;
        handler |= (self.handler_high as u64) << 32;
        handler
    }
}

macro_rules! impl_set_handler {
    ($h:ty) => {
        impl Entry<$h> {
            pub fn set_handler(&mut self, handler: $h) {
                self.set_handler_addr(handler as u64);
            }
        }
    };
}

impl_set_handler!(HandlerFunc);
impl_set_handler!(HandlerFuncErrorCode);
impl_set_handler!(SegmentNotPresentHandlerFunc);
impl_set_handler!(PageFaultHandlerFunc);
impl_set_handler!(DivergingHandlerFunc);
impl_set_handler!(DivergingHandlerFuncErrorCode);

impl<T> Debug for Entry<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let options = self.options;
        let selector = self.selector;
        let mut debug = f.debug_struct("Entry");

        debug.field("selector", &selector);
        debug.field("handler", &self.get_handler());
        debug.field("options", &options);
        debug.finish()
    }
}

bitflags! {
    pub struct Options: u16 {
        const RESERVED = 0xE00;
        const PRESENT = 0x8000;
        const INTERRUPTS = 0x100;
        const PRIVILEGE_THREE = 0x6000;
    }
}

impl Options {
    /// All bits zero except the ones that must be one
    fn zero() -> Self {
        Self::RESERVED
    }

    /// Create new set of options
    pub fn new(present: bool, disable: bool) -> Self {
        let mut options = Self::zero();
        options.set(Self::PRESENT, present);
        options.set(Self::INTERRUPTS, !disable);
        options
    }

    /// Set stack index level 0 = None, 1-7 valid stacks (IST)
    pub fn set_stack_index(&mut self, index: u16) {
        // valid range for stack index is 0 - 7
        assert!(index < 8);
        self.bits.set_bits(0..3, index + 1);
    }

    pub fn get_stack_index(&self) -> Option<u16> {
        match self.bits.get_bits(0..3) {
            0 => None,
            i => Some(i - 1),
        }
    }
}

impl Default for Options {
    /// Present and disable interrupts
    fn default() -> Self {
        Self::new(true, true)
    }
}

#[derive(Debug, Clone, Copy, From)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = 0,
    Keyboard = 1,
    Com1 = 4,
    Ide = 14,
    Error = 19,
    Spurious = 31,
}

impl From<usize> for InterruptIndex {
    fn from(num: usize) -> Self {
        match num {
            0 => Self::Timer,
            1 => Self::Keyboard,
            4 => Self::Com1,
            14 => Self::Ide,
            19 => Self::Error,
            31 => Self::Spurious,
            _ => unreachable!(),
        }
    }
}

impl From<InterruptIndex> for usize {
    fn from(index: InterruptIndex) -> Self {
        index as usize
    }
}

impl From<u8> for InterruptIndex {
    fn from(num: u8) -> Self {
        Self::from(num as usize)
    }
}

pub mod handlers {
    use crate::interrupts::errors::{ExceptionStackFrame, PageFaultErrorCode, SelectorError};
    use crate::interrupts::halt_loop;
    use crate::kprintln;

    /// 1
    pub extern "x86-interrupt" fn divide_by_zero(stack_frame: ExceptionStackFrame) {
        kprintln!("EXCEPTION: DIVIDE BY ZERO\n{:#?}", stack_frame);
    }

    /// 2
    pub extern "x86-interrupt" fn debug(stack_frame: ExceptionStackFrame) {
        kprintln!("EXCEPTION: DEBUG\n{:#?}", stack_frame);
    }

    /// 3
    pub extern "x86-interrupt" fn non_maskable_interrupt(stack_frame: ExceptionStackFrame) {
        kprintln!("EXCEPTION: NON MASKABLE INTERRUPT\n{:#?}", stack_frame);
    }

    /// 4
    pub extern "x86-interrupt" fn breakpoint(stack_frame: ExceptionStackFrame) {
        kprintln!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
    }

    /// 5
    pub extern "x86-interrupt" fn overflow(stack_frame: ExceptionStackFrame) {
        kprintln!("EXCEPTION: OVERFLOW\n{:#?}", stack_frame);
    }

    /// 6
    pub extern "x86-interrupt" fn bound_range_exceeded(stack_frame: ExceptionStackFrame) {
        kprintln!("EXCEPTION: BOUND RANGE EXCEEDED\n{:#?}", stack_frame);
    }

    /// 7
    pub extern "x86-interrupt" fn invalid_opcode(stack_frame: ExceptionStackFrame) {
        kprintln!("EXCEPTION: INVALID_OPCODE\n{:#?}", stack_frame);
    }

    /// 8
    pub extern "x86-interrupt" fn device_not_available(stack_frame: ExceptionStackFrame) {
        kprintln!("EXCEPTION: DEVICE NOT AVAILABLE\n{:#?}", stack_frame);
    }

    /// 9
    pub extern "x86-interrupt" fn double_fault(
        stack_frame: ExceptionStackFrame,
        _error_code: u64,
    ) -> ! {
        panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
    }

    /// 10
    pub extern "x86-interrupt" fn invalid_tss(stack_frame: ExceptionStackFrame, error_code: u64) {
        kprintln!(
            "EXCEPTION: INVALID_TSS\n{:#?}\nError Code: {}",
            stack_frame,
            error_code
        );
    }

    /// 11
    pub extern "x86-interrupt" fn segment_not_present(
        stack_frame: ExceptionStackFrame,
        error_code: SelectorError,
    ) {
        kprintln!(
            "EXCEPTION: SEGMENT_NOT_PRESENT\n{:#?}\n{:#?}",
            stack_frame,
            error_code
        );
    }

    /// 12
    pub extern "x86-interrupt" fn stack_segment_fault(
        stack_frame: ExceptionStackFrame,
        error_code: u64,
    ) {
        kprintln!(
            "EXCEPTION: STACK SEGMENT FAULT\n{:#?}\nError Code: {}",
            stack_frame,
            error_code
        );
    }

    /// 13
    pub extern "x86-interrupt" fn general_protection_fault(
        stack_frame: ExceptionStackFrame,
        error_code: u64,
    ) {
        kprintln!(
            "EXCEPTION: GENERAL PROTECTION FAULT\n{:#?}\nError Code: {}",
            stack_frame,
            error_code
        );
    }

    /// 14
    pub extern "x86-interrupt" fn page_fault(
        stack_frame: ExceptionStackFrame,
        error_code: PageFaultErrorCode,
    ) {
        use x86_64::registers::control::Cr2;

        kprintln!("EXCEPTION: PAGE FAULT");
        kprintln!("Accessed Address: {:?}", Cr2::read());
        kprintln!("Error Code: {:?}", error_code);
        kprintln!("{:#?}", stack_frame);
        halt_loop();
    }

    /// 15
    pub extern "x86-interrupt" fn x87_floating_point(stack_frame: ExceptionStackFrame) {
        kprintln!("EXCEPTION: x87 FLOATING POINT\n{:#?}", stack_frame);
    }

    /// 16
    pub extern "x86-interrupt" fn alignment_check(
        stack_frame: ExceptionStackFrame,
        error_code: u64,
    ) {
        kprintln!(
            "EXCEPTION: x87 FLOATING POINT\n{:#?}\nError Code: {}",
            stack_frame,
            error_code
        );
    }

    /// 17
    pub extern "x86-interrupt" fn machine_check(stack_frame: ExceptionStackFrame) -> ! {
        panic!("EXCEPTION: MACHINE CHECK\n{:#?}", stack_frame);
    }

    /// 18
    pub extern "x86-interrupt" fn simd_floating_point(stack_frame: ExceptionStackFrame) {
        kprintln!("EXCEPTION: SIMD FLOATING POINT\n{:#?}", stack_frame);
    }

    /// 19
    pub extern "x86-interrupt" fn virtualization(stack_frame: ExceptionStackFrame) {
        kprintln!("EXCEPTION: VIRTUALIZATION\n{:#?}", stack_frame);
    }

    /// 20
    pub extern "x86-interrupt" fn security_exception(
        stack_frame: ExceptionStackFrame,
        error_code: u64,
    ) {
        kprintln!(
            "EXCEPTION: VIRTUALIZATION\n{:#?}\nError Code: {}",
            stack_frame,
            error_code
        );
    }

    // Interrupt handler

    /// Timer Interrupt
    pub extern "x86-interrupt" fn timer(_stack_frame: ExceptionStackFrame) {
        lapic_eoi();
    }

    /// Keyboard Interrupt
    pub extern "x86-interrupt" fn keyboard(_stack_frame: ExceptionStackFrame) {
        use crate::io::keyboard::Keyboard;

        let keyboard = Keyboard::new();
        if let Some(key) = keyboard.get_scancode() {
            crate::io::keyboard::add_scancode(key);
        }

        lapic_eoi();
    }

    /// Ide Interrupt
    pub extern "x86-interrupt" fn ide(_stack_frame: ExceptionStackFrame) {
        crate::disk::interrupt_handler();
        lapic_eoi();
    }

    /// Send a end of interrupt to the local apic
    pub fn lapic_eoi() {
        unsafe { (*crate::io::LAPIC.as_mut_ptr()).end_of_interrupt() };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::size_of;

    /// Make sure the entry struct is getting correctly packed
    #[test_case]
    fn entry_struct_size() {
        assert_eq!(size_of::<Entry<HandlerFunc>>(), 16);
    }

    /// Make sure the idt struct is getting correctly packed
    #[test_case]
    fn idt_struct_size() {
        assert_eq!(size_of::<InterruptDescriptorTable>(), 16 * 256);
    }
}
