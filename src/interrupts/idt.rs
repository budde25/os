use bit_field::BitField;
use core::fmt::{self, Debug, Formatter};

#[allow(dead_code)]
pub enum PrivilegeLevel {
    Ring0 = 0,
    Ring1 = 1,
    Ring2 = 2,
    Ring3 = 3,
}

pub type HandlerFunc = extern "C" fn() -> !;

/// Interrupt Descriptor Table
#[derive(Debug)]
pub struct InterruptDescriptorTable([Entry; 16]);

impl InterruptDescriptorTable {
    pub fn new() -> Self {
        Self([Entry::empty(); 16])
    }

    pub fn set_handler(&mut self, index: u8, handler: HandlerFunc) {
        let segment = SegmentSelector::code_segment();
        self.0[index as usize] = Entry::new(segment, handler);
    }

    pub fn load(&'static self) {
        use core::mem::size_of;
        let mut ptr = DescriptorTablePointer {
            base: self as *const _ as u64,
            limit: (size_of::<Self>() - 1) as u16,
        };
        let gdt = &mut ptr;
        unsafe {
            asm!("lidt [{}]", in(reg) gdt, options(nostack));
        }
        crate::println!("{:#?}", self);
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
struct DescriptorTablePointer {
    base: u64,  // Base addr
    limit: u16, // the inclusive limit from the base
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct Entry {
    handler_low: u16,          // offset bits 0..15
    selector: SegmentSelector, // a code segment selector in GDT or LDT
    options: Options,          // type and attributes
    handler_middle: u16,       // offset bits 16..31
    handler_high: u32,         // offset bits 32..63
    zero: u32,                 // reserved
}

impl Entry {
    /// Create a new present entry
    fn new(selector: SegmentSelector, handler: HandlerFunc) -> Self {
        let pointer = handler as u64;
        Self {
            selector,
            handler_low: pointer as u16,
            handler_middle: (pointer >> 16) as u16,
            handler_high: (pointer >> 32) as u32,
            options: Options::default(),
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
            options: Options::zero(),
            zero: 0,
        }
    }

    fn get_handler(&self) -> u64 {
        let mut handler: u64 = self.handler_low as u64;
        handler = handler | ((self.handler_middle as u64) << 16);
        handler = handler | ((self.handler_high as u64) << 32);
        handler
    }
}

impl Debug for Entry {
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
