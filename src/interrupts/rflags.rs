use bitflags::bitflags;
use core::arch::asm;

bitflags! {
    #[repr(C)]
    pub struct RFlags: u64 {
       const CARRY = 0x1;
       const RESERVED_1 = 0x2;
       const PARITY = 0x4;
       const RESERVED_2 = 0x8;
       const AUX_CARRY = 0x10;
       const RESERVED_3 = 0x20;
       const ZERO = 0x40;
       const SIGN = 0x80;
       const TRAP = 0x100;
       const INTERRUPT_ENABLE = 0x200;
       const DIRECTION = 0x400;
       const OVERFLOW = 0x800;
       const IO_PRIVILEGE_THREE = 0x3000;
       const NESTED_TASK = 0x4000;
       const RESERVED_4 = 0x8000;
       const RESUME = 0x10000;
       const VIRTUAL = 0x20000;
       const ALIGNMENT_ACCESS = 0x40000;
       const VIRTUAL_INTERRUPT = 0x80000;
       const VIRTUAL_INTERRUPT_PENDING = 0x100000;
       const ID = 0x200000;
       const RESERVED_5 = 0xFFFFFFFFFFC00000;
    }
}

impl RFlags {
    pub fn read() -> Self {
        let flags: u64;
        unsafe {
            asm!("pushfq; pop {}", out(reg) flags, options(nomem, preserves_flags));
        }
        Self::from_bits_truncate(flags)
    }
}
