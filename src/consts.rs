/// The size of a single PML4
pub const PML4_SIZE: usize = 0x0000_0080_0000_0000;
pub const PML4_MASK: usize = 0x0000_ff80_0000_0000;

/// Offset of kernel
pub const KERNEL_OFFSET: usize = 0xFFFF_8000_0000_0000;
pub const KERNEL_PML4: usize = (KERNEL_OFFSET & PML4_MASK) / PML4_SIZE;
