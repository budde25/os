pub use x86_64::consts::{SIZE_1KIB, SIZE_1MIB};

// HEAP
pub const HEAP_START: u64 = 0x100_0000;
pub const HEAP_SIZE: u64 = 2 * SIZE_1MIB; // 2 MiB

// KHEAP
pub const KHEAP_START: u64 = HEAP_START + HEAP_SIZE;

// IRQ's
pub const IRQ_0: u8 = 32;

// IRQ offset table
pub use crate::interrupts::idt::InterruptIndex as IRQ;

//
pub const ROOTINO: usize = 2; // root i node number
pub const SSIZE: usize = 512; // sector size
pub const BSIZE: usize = 512 * 2; // buffer size
