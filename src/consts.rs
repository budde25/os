/// Offset of kernel
pub const KERNEL_OFFSET: u64 = 0xFFFF_8000_0000_0000;

// HEAP
pub const HEAP_START: u64 = 0x1000000;
pub const HEAP_SIZE: u64 = 100 * 1024; // 100 KiB

// KHEAP
pub const KHEAP: u64 = HEAP_SIZE + HEAP_SIZE;
pub const KHEAP_SIZE: u64 = 256 * 4096;

// SIZES
pub const SIZE_1KIB: u64 = 0x1000;
pub const SIZE_1MIB: u64 = 0x10_0000;

// IRQ's
pub const IRQ_0: u8 = 32;

// IRQ offset table
pub use crate::interrupts::idt::InterruptIndex as IRQ;
