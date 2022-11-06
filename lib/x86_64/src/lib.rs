#![no_std]

pub mod tables;
pub mod paging;
pub mod registers;
pub mod consts;
mod address;

pub use address::{PhysicalAddress, VirtualAddress};

/// Offset of kernel
pub const KERNEL_OFFSET: u64 = 0xFFFF_8000_0000_0000;


