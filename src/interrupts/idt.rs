use super::{
    DescriptorTablePointer, DivergingHandlerFunc, DivergingHandlerFuncErrorCode, HandlerFunc,
    HandlerFuncErrorCode, SegmentSelector,
};
use bit_field::BitField;
use core::{
    fmt::{self, Debug, Formatter},
    marker::PhantomData,
};

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

#[derive(Clone, Copy)]
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
    /// Set a presnt entry with handler
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
            pub fn set_handler(&mut self, handler: $h) {
                self.set_handler_addr(handler as u64);
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
    pub fn set_priviledge_level(&mut self, dpl: u16) {
        self.0.set_bits(13..15, dpl);
    }

    pub fn get_priviledge_level(&self) -> u16 {
        self.0.get_bits(13..15)
    }

    /// Set stack index level 0 = None, 1-7 valid stacks (IST)
    pub fn set_stack_index(&mut self, index: u16) {
        self.0.set_bits(0..3, index);
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
        let privilege_level = self.get_priviledge_level();
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
