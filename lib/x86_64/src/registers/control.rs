use crate::paging::phys_frame::PhysFrame;
use crate::{PhysicalAddress, VirtualAddress};
use bitflags::bitflags;
use core::arch::asm;

bitflags! {
    pub struct Cr0: u64 {
        const PROTECTION_MODE_ENABLED = 1;
        const MONITOR_COPROCESSOR = 1 << 1;
        const EMULATION = 1 << 2;
        const TASK_SWITCHED = 1 << 3;
        const EXTENSION_TYPE = 1 << 4;
        const NUMERIC_ERROR = 1 << 5;
        const WRITE_PROTECT = 1 << 16;
        const ALIGNMENT_MASK = 1 << 18;
        const NOT_WRITE_THROUGH = 1 << 29;
        const CACHE_DISABLE = 1 << 30;
        const PAGING = 1 << 31;
    }
}

#[derive(Debug)]
pub struct Cr2;

impl Cr2 {
    pub fn read() -> VirtualAddress {
        let value: u64;
        unsafe { asm!("mov {}, cr2", out(reg) value, options(nomem, nostack, preserves_flags)) };
        VirtualAddress::new(value)
    }
}

#[derive(Debug)]
pub struct Cr3 {
    frame: PhysFrame,
    flags: Cr3Flags,
}

bitflags! {
    pub struct Cr3Flags: u64 {
        const PAGE_LEVEL_WRITE_THROUGH = 1 << 3;
        const PAGE_LEVEL_CACHE_DISABLE = 1 << 4;
    }
}

impl Cr3 {
    pub fn read() -> Self {
        let value: u64;
        unsafe { asm!("mov {}, cr3", out(reg) value, options(nomem, nostack, preserves_flags)) };
        let flags = Cr3Flags::from_bits_truncate(value);
        let address = PhysicalAddress::new(value & 0x_000f_ffff_ffff_f000);
        let frame = PhysFrame::containing_address(address);
        Self { frame, flags }
    }

    pub unsafe fn write(frame: PhysFrame, flags: Cr3Flags) {
        let addr = frame.address();
        let value = u64::from(addr) | flags.bits();
        asm!("mov cr3, {}", in(reg) value, options(nostack, preserves_flags));
    }

    pub fn frame(&self) -> PhysFrame {
        self.frame
    }

    pub fn flags(&self) -> Cr3Flags {
        self.flags
    }
}

bitflags! {
    pub struct Cr4: u64 {
        const VME = 1;
        const PVI = 1 << 1;
        const TSD = 1 << 2;
        const DE = 1 << 3;
        const PSE = 1 << 4;
        const PAE = 1 << 5;
        const MCE = 1 << 6;
        const PGE = 1 << 7;
        const PCE = 1 << 8;
        const OSFXSR = 1 << 9;
        const OSXMMEXCPT = 1 << 10;
        const UMIP = 1 << 11;
        const VMXE = 1 << 13;
        const SMXE = 1 << 14;
        const PCIDE = 1 << 17;
        const OXXSAVE = 1 << 18;
        const SMEP = 1 << 20;
        const SMAP = 1 << 21;
        const PKE = 1 << 22;
        const CET = 1 << 23;
        const PKS = 1 << 24;
    }
}

impl Cr4 {
    pub fn read() -> Self {
        let value: u64;
        unsafe { asm!("mov {}, cr4", out(reg) value, options(nomem, nostack, preserves_flags)) };
        Self::from_bits_truncate(value)
    }
}
