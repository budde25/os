/// Offset of kernel
pub const KERNEL_OFFSET: u64 = 0xFFFF_8000_0000_0000;

// HEAP
pub const HEAP_START: u64 = 0x100_0000;
pub const HEAP_SIZE: u64 = 2 * SIZE_1MIB; // 2 MiB

// KHEAP
pub const KHEAP_START: u64 = HEAP_START + HEAP_SIZE;

// SIZES
pub const SIZE_1KIB: u64 = 0x1000;
pub const SIZE_1MIB: u64 = 0x10_0000;

// IRQ's
pub const IRQ_0: u8 = 32;

// IRQ offset table
pub use crate::interrupts::idt::InterruptIndex as IRQ;

//
pub const ROOTINO: usize = 1; // root i node number
pub const BSIZE: usize = 512;
