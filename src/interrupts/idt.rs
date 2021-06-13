use super::{DescriptorTablePointer, SegmentSelector};
use bit_field::BitField;
use core::{
    fmt::{self, Debug, Formatter},
    marker::PhantomData,
};
use handlers::ExceptionStackFrame;

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
    pub segment_not_present: Entry<HandlerFuncErrorCode>,
    pub stack_segment_fault: Entry<HandlerFuncErrorCode>,
    pub general_protection_fault: Entry<HandlerFuncErrorCode>,
    pub page_fault: Entry<HandlerFuncErrorCode>,
    reserved_1: Entry<HandlerFunc>,
    pub x87_floating_point: Entry<HandlerFunc>,
    pub alignment_check: Entry<HandlerFuncErrorCode>,
    pub machine_check: Entry<DivergingHandlerFunc>,
    pub simd_floating_point: Entry<HandlerFunc>,
    pub virtualization: Entry<HandlerFunc>,
    reserved_2: [Entry<HandlerFunc>; 9],
    pub security_exception: Entry<HandlerFuncErrorCode>,
    reserved_3: Entry<HandlerFunc>,
    interrupts: [Entry<HandlerFunc>; 256 - 32],
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

#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct Entry<T> {
    handler_low: u16,          // offset bits 0..15
    selector: SegmentSelector, // a code segment selector in GDT or LDT
    options: Options,          // type and attributes
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
            selector: SegmentSelector::zero(),
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
        handler = handler | ((self.handler_middle as u64) << 16);
        handler = handler | ((self.handler_high as u64) << 32);
        handler
    }
}

macro_rules! impl_set_handler {
    ($h:ty) => {
        impl Entry<$h> {
            pub fn set_handler(&mut self, handler: $h) -> Options {
                self.set_handler_addr(handler as u64);
                self.options
            }
        }
    };
}

impl_set_handler!(HandlerFunc);
impl_set_handler!(HandlerFuncErrorCode);
//impl_set_handler!(PageFaultHandlerFunc);
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

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Options(u16);

#[allow(dead_code)]
impl Options {
    /// All bits zero except the ones that must be one
    fn zero() -> Self {
        let mut options: u16 = 0;
        options.set_bits(9..12, 0b111); // this bits must be 1
        Self(options)
    }

    /// Create new set of options
    pub fn new(present: bool, disable: bool) -> Self {
        let mut options = Self::zero();
        options.set_present(present);
        options.disable_interrupts(disable);
        options
    }

    /// Set present
    pub fn set_present(&mut self, present: bool) {
        self.0.set_bit(15, present);
    }

    pub fn is_present(&self) -> bool {
        self.0.get_bit(15)
    }

    /// Disable interrupts
    pub fn disable_interrupts(&mut self, disable: bool) {
        self.0.set_bit(8, !disable);
    }

    pub fn is_interrupts(&self) -> bool {
        self.0.get_bit(8)
    }

    /// Set the privilege level 0-3
    pub fn set_privilege_level(&mut self, dpl: u16) {
        self.0.set_bits(13..15, dpl);
    }

    pub fn get_privilege_level(&self) -> u16 {
        self.0.get_bits(13..15)
    }

    /// Set stack index level 0 = None, 1-7 valid stacks (IST)
    pub fn set_stack_index(&mut self, index: u16) {
        self.0.set_bits(0..3, index + 1);
    }

    pub fn get_stack_index(&self) -> u16 {
        self.0.get_bits(0..3)
    }
}

impl Default for Options {
    /// Present and disable interrupts
    fn default() -> Self {
        Self::new(true, true)
    }
}

impl Debug for Options {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let ist = self.get_stack_index();
        let privilege_level = self.get_privilege_level();
        let present = self.is_present();
        let interrupts = self.is_interrupts();
        let mut debug = f.debug_struct("Options");

        debug.field("ist", &ist);
        debug.field("privilege_level", &privilege_level);
        debug.field("present", &present);
        debug.field("interrupts", &interrupts);
        debug.finish()
    }
}

pub mod handlers {
    use crate::println;

    #[derive(Debug, Clone, Copy)]
    #[repr(C)]
    pub struct ExceptionStackFrame {
        instruction_pointer: u64,
        code_segment: u64,
        cpu_flags: u64,
        stack_pointer: u64,
        stack_segment: u64,
    }

    /// 1
    pub extern "x86-interrupt" fn divide_by_zero(stack_frame: ExceptionStackFrame) {
        println!("EXCEPTION: DIVIDE BY ZERO\n{:#?}", stack_frame);
    }

    /// 2
    pub extern "x86-interrupt" fn debug(stack_frame: ExceptionStackFrame) {
        println!("EXCEPTION: DEBUG\n{:#?}", stack_frame);
    }

    /// 3
    pub extern "x86-interrupt" fn non_maskable_interrupt(stack_frame: ExceptionStackFrame) {
        println!("EXCEPTION: NON MASKABLE INTERRUPT\n{:#?}", stack_frame);
    }

    /// 4
    pub extern "x86-interrupt" fn breakpoint(stack_frame: ExceptionStackFrame) {
        println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
    }

    /// 5
    pub extern "x86-interrupt" fn overflow(stack_frame: ExceptionStackFrame) {
        println!("EXCEPTION: OVERFLOW\n{:#?}", stack_frame);
    }

    /// 6
    pub extern "x86-interrupt" fn bound_range_exceeded(stack_frame: ExceptionStackFrame) {
        println!("EXCEPTION: BOUND RANGE EXCEEDED\n{:#?}", stack_frame);
    }

    /// 7
    pub extern "x86-interrupt" fn invalid_opcode(stack_frame: ExceptionStackFrame) {
        println!("EXCEPTION: INVALID_OPCODE\n{:#?}", stack_frame);
    }

    /// 8
    pub extern "x86-interrupt" fn device_not_available(stack_frame: ExceptionStackFrame) {
        println!("EXCEPTION: DEVICE NOT AVAILABLE\n{:#?}", stack_frame);
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
        println!(
            "EXCEPTION: INVALID_TSS\n{:#?}\nError Code:{}",
            stack_frame, error_code
        );
    }

    /// 11
    pub extern "x86-interrupt" fn segment_not_present(
        stack_frame: ExceptionStackFrame,
        error_code: u64,
    ) {
        println!(
            "EXCEPTION: SEGMENT_NOT_PRESENT\n{:#?}\nError Code:{}",
            stack_frame, error_code
        );
    }

    /// 12
    pub extern "x86-interrupt" fn stack_segment_fault(
        stack_frame: ExceptionStackFrame,
        error_code: u64,
    ) {
        println!(
            "EXCEPTION: STACK SEGMENT FAULT\n{:#?}\nError Code:{}",
            stack_frame, error_code
        );
    }

    /// 13
    pub extern "x86-interrupt" fn general_protection_fault(
        stack_frame: ExceptionStackFrame,
        error_code: u64,
    ) {
        println!(
            "EXCEPTION: GENERAL PROTECTION FAULT\n{:#?}\nError Code:{}",
            stack_frame, error_code
        );
    }

    /// 14
    pub extern "x86-interrupt" fn page_fault(stack_frame: ExceptionStackFrame, error_code: u64) {
        println!(
            "EXCEPTION: PAGE FAULT\n{:#?}\nError Code:{}",
            stack_frame, error_code
        );
    }

    /// 15
    pub extern "x86-interrupt" fn x87_floating_point(stack_frame: ExceptionStackFrame) {
        println!("EXCEPTION: x87 FLOATING POINT\n{:#?}", stack_frame);
    }

    /// 16
    pub extern "x86-interrupt" fn alignment_check(
        stack_frame: ExceptionStackFrame,
        error_code: u64,
    ) {
        println!(
            "EXCEPTION: x87 FLOATING POINT\n{:#?}\nError Code:{}",
            stack_frame, error_code
        );
    }

    /// 17
    pub extern "x86-interrupt" fn machine_check(stack_frame: ExceptionStackFrame) -> ! {
        panic!("EXCEPTION: MACHINE CHECK\n{:#?}", stack_frame);
    }

    /// 18
    pub extern "x86-interrupt" fn simd_floating_point(stack_frame: ExceptionStackFrame) {
        println!("EXCEPTION: SIMD FLOATING POINT\n{:#?}", stack_frame);
    }

    /// 19
    pub extern "x86-interrupt" fn virtualization(stack_frame: ExceptionStackFrame) {
        println!("EXCEPTION: VIRTUALIZATION\n{:#?}", stack_frame);
    }

    /// 20
    pub extern "x86-interrupt" fn security_exception(
        stack_frame: ExceptionStackFrame,
        error_code: u64,
    ) {
        println!(
            "EXCEPTION: VIRTUALIZATION\n{:#?}\nError Code:{}",
            stack_frame, error_code
        );
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
