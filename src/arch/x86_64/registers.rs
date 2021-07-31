use crate::address::virt::VirtualAddress;
use bitflags::bitflags;

#[derive(Debug)]
pub struct Cr2;

impl Cr2 {
    pub fn read() -> VirtualAddress {
        let value: u64;
        unsafe { asm!("mov {}, cr2", out(reg) value, options(nomem, nostack, preserves_flags)) };
        VirtualAddress::new(value)
    }
}
