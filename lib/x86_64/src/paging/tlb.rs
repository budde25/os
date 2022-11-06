use crate::VirtualAddress;
use core::arch::asm;

/// flush a specific virtual addresss from the tlb
pub fn flush(addr: VirtualAddress) {
    unsafe {
        asm!("invlpg [{}]", in(reg) u64::from(addr), options(nostack, preserves_flags));
    }
}

/// Invalidate the TLB completely by reloading the CR3 register.
pub fn flush_all() {
    use crate::registers::control::Cr3;

    let cr3 = Cr3::read();
    unsafe { Cr3::write(cr3.frame(), cr3.flags()) }
}
