use bit_field::BitField;
use lazy_static::lazy_static;

enum PrivilegeLevel {
    Ring0 = 0,
    Ring1 = 1,
    Ring2 = 2,
    Ring3 = 3,
}

pub type HandlerFunc = extern "C" fn() -> !;

/// Interrupt Descriptor Table
pub struct IDT([IDTDescriptor; 16]);

impl IDT {
    pub fn new() -> Self {
        Self([IDTDescriptor::empty(); 16])
    }

    pub fn set_handler(&mut self, index: u8, handler: HandlerFunc) {
        let mut segment = SegmentSelector::zero();
        segment.set_index_code_segment();
        self.0[index as usize] = IDTDescriptor::new(segment, handler);
    }

    pub fn load(&'static self) {
        use core::mem::size_of;
        let mut ptr = DescriptorTablePointer {
            base: self as *const _ as u64,
            limit: (size_of::<Self>() - 1) as u16,
        };

        unsafe {
            asm!("lidt [{}]", in(reg) &mut ptr, options(nostack));
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
struct DescriptorTablePointer {
    base: u64,  // Base addr
    limit: u16, // the inclusive limit from the base
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
struct IDTDescriptor {
    handler_low: u16,          // offset bits 0..15
    selector: SegmentSelector, // a code segment selector in GDT or LDT
    options: IDTOptions,       // type and attributes
    handler_middle: u16,       // offset bits 16..31
    handler_high: u32,         // offset bits 32..63
    zero: u32,                 // reserved
}

impl IDTDescriptor {
    /// Create a new present entry
    fn new(selector: SegmentSelector, handler: HandlerFunc) -> Self {
        let pointer = handler as u64;
        Self {
            selector,
            handler_low: pointer as u16,
            handler_middle: (pointer >> 16) as u16,
            handler_high: (pointer >> 32) as u32,
            options: IDTOptions::default(),
            zero: 0,
        }
    }

    /// Create a non present entry
    fn empty() -> Self {
        Self {
            selector: SegmentSelector::zero(),
            handler_low: 0,
            handler_middle: 0,
            handler_high: 0,
            options: IDTOptions::zero(),
            zero: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
struct SegmentSelector(u16);

impl SegmentSelector {
    fn zero() -> Self {
        Self(0)
    }

    fn new(index: u16, level: PrivilegeLevel) -> Self {
        let mut selector = Self::zero();
        selector.set_priviledge_level(level);
        selector.set_index(index);
        selector
    }

    fn set_priviledge_level(&mut self, level: PrivilegeLevel) {
        self.0.set_bits(0..1, level as u16);
    }

    fn set_index(&mut self, index: u16) {
        self.0.set_bits(3..15, index);
    }

    fn set_index_code_segment(&mut self) {
        let segment: u16;
        unsafe { asm!("mov {0:x}, cs", out(reg) segment, options(nostack, nomem)) };
        self.set_index(segment);
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
struct IDTOptions(u16);

impl IDTOptions {
    /// All bits zero except the ones that must be one
    fn zero() -> Self {
        let mut options: u16 = 0;
        options.set_bits(9..12, 0b111); // this bits must be 1
        Self(options)
    }

    /// Create new set of options
    fn new(present: bool, disable: bool) -> Self {
        let mut options = Self::zero();
        options.set_present(present);
        options.set_present(disable);
        options
    }

    /// Set present
    fn set_present(&mut self, present: bool) {
        self.0.set_bit(15, present);
    }

    /// Disable interrupts
    fn disable_interrupts(&mut self, disable: bool) {
        self.0.set_bit(8, !disable);
    }

    /// Set the privilege level 0-3
    fn set_priviledge_level(&mut self, dpl: u16) {
        self.0.set_bits(13..15, dpl);
    }

    /// Set stack index level 0 = None, 1-7 valid stacks
    fn set_stack_index(&mut self, index: u16) {
        self.0.set_bits(0..3, index);
    }
}

impl Default for IDTOptions {
    /// Present and disable interrupts
    fn default() -> Self {
        Self::new(true, true)
    }
}
