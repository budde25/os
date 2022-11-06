// SIZES
pub const SIZE_1KIB: u64 = 0x1000;
pub const SIZE_1MIB: u64 = 0x10_0000;

// TODO: DOESN'T BELONG HERE
pub const HEAP_START: u64 = 0x100_0000;
pub const HEAP_SIZE: u64 = 2 * SIZE_1MIB; // 2 MiB
pub const KHEAP_START: u64 = HEAP_START + HEAP_SIZE;