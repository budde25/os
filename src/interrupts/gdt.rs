use crate::interrupts::tss::TaskStateSegment;
use crate::interrupts::DescriptorTablePointer;

use super::{PrivilegeLevel, SegmentSelector};
use bit_field::BitField;
use bitflags::bitflags;
use core::fmt::{self, Debug, Formatter};

#[derive(Debug, Clone, Copy, Hash)]
#[repr(C, packed)]
pub struct GlobalDescriptorTable([Entry; 7]);

impl GlobalDescriptorTable {
    pub fn new() -> Self {
        Self([Entry::empty(); 7])
    }

    pub fn set_entry(&mut self, index: u8, entry: Entry) -> SegmentSelector {
        self.0[index as usize] = entry;
        // TODO support more then just ring0
        SegmentSelector::new(index as u16, PrivilegeLevel::Ring0)
    }

    fn pointer(&self) -> DescriptorTablePointer {
        use core::mem::size_of;
        DescriptorTablePointer {
            base: self.0.as_ptr() as u64,
            limit: (size_of::<Self>() - 1) as u16,
        }
    }

    pub fn load(&'static self) {
        let ptr = self.pointer();
        unsafe {
            asm!("lgdt [{}]", in(reg) &ptr, options(nostack, readonly, preserves_flags));
        }
    }
}

#[inline]
pub unsafe fn load_cs(segment: SegmentSelector) {
    asm!(
        "push {sel}",
        "lea {tmp}, [1f + rip]",
        "push {tmp}",
        "retfq",
        "1:",
        sel = in(reg) u64::from(segment.0),
        tmp = lateout(reg) _,
        options(preserves_flags),
    );
}

#[inline]
pub unsafe fn load_tss(segment: SegmentSelector) {
    asm!("ltr {0:x}", in(reg) segment.0, options(nomem, nostack, preserves_flags));
}

#[derive(Clone, Copy, Hash)]
#[repr(C, packed)]
pub struct Entry {
    limit_1: u16,
    base_1: u16,
    base_2: u8,
    flags: Flags, // limit_2 is also in there 8:11
    base_3: u8,
}

impl Entry {
    fn empty() -> Self {
        Self::new(0, Flags::empty())
    }

    pub fn new(base: u32, flags: Flags) -> Self {
        // limit ignore in 64 so set it all to 1
        let limit = u32::MAX;
        let mut flags = flags;
        flags.set_limit_2((limit >> 16) as u16);

        Self {
            limit_1: limit as u16,
            base_1: base as u16,
            base_2: (base >> 16) as u8,
            flags,
            base_3: (base >> 24) as u8,
        }
    }

    pub fn tss(tss: &'static TaskStateSegment) -> (Self, Self) {
        use core::mem::size_of;

        let ptr = tss as *const _ as u64;

        let mut low_flags = Flags::TSS;
        low_flags.set_limit_2(0b1001);

        let low = Self {
            limit_1: (size_of::<TaskStateSegment>() - 1) as u16,
            base_1: ptr.get_bits(0..16) as u16,
            base_2: ptr.get_bits(16..24) as u8,
            flags: low_flags,
            base_3: ptr.get_bits(24..32) as u8,
        };

        let high = Self {
            limit_1: ptr.get_bits(32..48) as u16,
            base_1: ptr.get_bits(48..64) as u16,
            base_2: 0,
            flags: Flags::empty(),
            base_3: 0,
        };
        (low, high)
    }
}

impl Debug for Entry {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut limit: u32 = self.limit_1 as u32;
        let flags = self.flags;
        limit = limit | ((flags.get_limit_2() as u32) << 16);
        let mut base: u32 = self.base_1 as u32;
        base = base | ((self.base_2 as u32) << 16);
        base = base | ((self.base_3 as u32) << 24);
        let flags = self.flags;
        let mut debug = f.debug_struct("Entry");
        debug.field("base", &base);
        debug.field("limit", &limit);
        debug.field("flags", &flags);
        debug.finish()
    }
}

bitflags! {
    /// Flag Bytes
    ///
    /// Bit(s) | Name
    /// -----------
    /// 0      | Accessed
    /// 1      | Readable/Writeable
    /// 2      | Direction
    /// 3      | Executable
    /// 4      | Descriptor type
    /// 5, 6   | Privilege level
    /// 7      | Present
    /// 8:11   | Limit_2
    /// 13     | Code descriptor
    /// 14     | Size
    /// 15     | Granularity
    pub struct Flags: u16 {
        const ACCESSED  = 0x1; // Accessed
        const WRITABLE  = 0x2; // Writeable
        const CONFORMING = 0x4; // Conforming / Expand down
        const EXECUTABLE = 0x8; // Executable
        const DESCTYPE  = 0x10; // Descriptor type (0 for system, 1 for code/data)
        const PRESENT   = 0x80; // Present
        const SAVL      = 0x1000; // Available for system use
        const LONG      = 0x2000; // Long mode
        const SIZE      = 0x4000; // Size (0 for 16-bit, 1 for 32)
        const GRANULARITY = 0x8000; // Granularity (0 for 1B - 1MB, 1 for 4KB - 4GB)
        const PRIVILEGE_THREE = 0x60; // Privilege level 3
        const LIMIT_TWO   = 0xF00;
    }
}

#[allow(dead_code)]
impl Flags {
    const COMMON: Self = Self::from_bits_truncate(
        Self::ACCESSED.bits
            | Self::WRITABLE.bits
            | Self::DESCTYPE.bits
            | Self::PRESENT.bits
            | Self::GRANULARITY.bits
            | Self::LIMIT_TWO.bits,
    );

    pub const CODE_PL_ZERO: Self =
        Self::from_bits_truncate(Self::COMMON.bits | Self::EXECUTABLE.bits | Self::LONG.bits);

    pub const DATA_PL_ZERO: Self = Self::from_bits_truncate(Self::COMMON.bits | Self::SIZE.bits);

    pub const CODE_PL_THREE: Self =
        Self::from_bits_truncate(Self::CODE_PL_ZERO.bits | Self::PRIVILEGE_THREE.bits);

    pub const DATA_PL_THREE: Self =
        Self::from_bits_truncate(Self::DATA_PL_ZERO.bits | Self::PRIVILEGE_THREE.bits);

    pub const TSS: Self = Self::from_bits_truncate(
        Self::EXECUTABLE.bits | Self::ACCESSED.bits | Self::PRESENT.bits | Self::SIZE.bits,
    );

    fn get_limit_2(&self) -> u16 {
        self.bits.get_bits(8..12)
    }

    fn set_limit_2(&mut self, value: u16) {
        if value < 0b1111 {
            self.bits.set_bits(8..12, value);
        } else {
            self.bits.set_bits(8..12, 0b1111);
        }
    }
}

impl From<u16> for Flags {
    fn from(num: u16) -> Self {
        unsafe { Flags::from_bits_unchecked(num) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::{size_of, transmute};

    /// Make sure the entry struct is getting correctly packed
    #[test_case]
    fn entry_struct_size() {
        assert_eq!(size_of::<Entry>(), 8);
    }

    /// Make sure the gdt struct is getting correctly packed
    #[test_case]
    fn gdt_struct_size() {
        assert_eq!(size_of::<GlobalDescriptorTable>(), 8 * 7);
    }

    /// Linux defaults
    #[test_case]
    #[rustfmt::skip]
    fn linux_defaults() {
        let code_pl_zero: u64 = unsafe{ transmute(Entry::new(0, Flags::CODE_PL_ZERO)) };
        assert_eq!(code_pl_zero, 0x00af9b000000ffff);

        let data_pl_zero: u64 = unsafe{ transmute(Entry::new(0, Flags::DATA_PL_ZERO)) };
        assert_eq!(data_pl_zero, 0x00cf93000000ffff);

        let code_pl_three: u64 = unsafe{ transmute(Entry::new(0, Flags::CODE_PL_THREE)) };
        assert_eq!(code_pl_three, 0x00affb000000ffff);
        
        let data_pl_three: u64 = unsafe{ transmute(Entry::new(0, Flags::DATA_PL_THREE)) };
        assert_eq!(data_pl_three, 0x00cff3000000ffff);
    }
}
