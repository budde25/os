use crate::interrupts::DescriptorTablePointer;

use super::PrivilegeLevel;
use bit_field::BitField;
use core::fmt::{self, Debug, Formatter};

#[derive(Debug, Clone, Copy, Hash)]
#[repr(C, packed)]
pub struct GlobalDescriptorTable([Entry; 5]);

impl GlobalDescriptorTable {
    pub fn new() -> Self {
        Self([Entry::empty(); 5])
    }

    pub fn set_entry(&mut self, index: u8, entry: Entry) {
        self.0[index as usize] = entry;
    }

    pub fn load(&'static self) {
        use core::mem::size_of;
        let mut ptr = DescriptorTablePointer {
            base: self as *const _ as u64,
            limit: (size_of::<Self>() - 1) as u16,
        };
        let gdt = &mut ptr;
        unsafe {
            asm!("lgdt [{}]", in(reg) gdt, options(nostack));
        }
    }
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
        Self::new(0, 0, Flags::zero())
    }

    pub fn new(base: u32, limit: u32, flags: Flags) -> Self {
        let mut f = flags;
        f.set_limit_2((limit >> 16) as u16);
        Self {
            limit_1: limit as u16,
            base_1: base as u16,
            base_2: (base >> 16) as u8,
            flags: f,
            base_3: (base >> 24) as u8,
        }
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
#[derive(Clone, Copy, Hash)]
#[repr(transparent)]
pub struct Flags(u16);

#[allow(dead_code)]
impl Flags {
    /// An zero entry
    fn zero() -> Self {
        Self(0)
    }

    pub fn code_ring_zero() -> Self {
        let mut flags = Self::zero();
        flags.set_descriptor_type(true);
        flags.set_present(true);
        flags.set_granularity(true);
        flags.set_granularity(true);
        flags.set_executable(true);
        flags.set_read_write(true);
        flags
    }

    pub fn data_ring_zero() -> Self {
        let mut flags = Self::zero();
        flags.set_descriptor_type(true);
        flags.set_present(true);
        flags.set_granularity(true);
        flags.set_granularity(true);
        flags.set_read_write(true);
        flags
    }

    fn set_granularity(&mut self, granularity: bool) {
        self.0.set_bit(15, granularity);
    }

    fn is_granularity(&self) -> bool {
        self.0.get_bit(15)
    }

    fn set_size(&mut self, size: bool) {
        self.0.set_bit(14, size);
    }

    fn is_size(&self) -> bool {
        self.0.get_bit(14)
    }

    /// reserved for data segments
    fn set_code_descriptor(&mut self, code_descriptor: bool) {
        if code_descriptor {
            self.0.set_bit(13, true);
            self.set_size(false);
        } else {
            self.0.set_bit(13, false);
        }
    }

    fn is_code_descriptor(&self) -> bool {
        self.0.get_bit(13)
    }

    fn set_limit_2(&mut self, limit_2: u16) {
        self.0.set_bits(8..12, limit_2);
    }

    fn get_limit_2(&self) -> u16 {
        self.0.get_bits(8..12)
    }

    pub fn set_present(&mut self, present: bool) {
        self.0.set_bit(7, present);
    }

    fn is_present(&self) -> bool {
        self.0.get_bit(7)
    }

    pub fn set_priviledge_level(&mut self, dpl: PrivilegeLevel) {
        self.0.set_bits(5..7, dpl as u16);
    }

    fn get_priviledge_level(&self) -> u16 {
        self.0.get_bits(5..7)
    }

    /// Should be true for code/data else false
    fn set_descriptor_type(&mut self, descriptor_type: bool) {
        self.0.set_bit(4, descriptor_type);
    }

    /// Returns true if it a code/data false otherwise
    fn is_descriptor_type(&self) -> bool {
        self.0.get_bit(4)
    }

    /// Should be true for code, false for data
    fn set_executable(&mut self, executable: bool) {
        self.0.set_bit(3, executable);
    }

    fn is_executable(&self) -> bool {
        self.0.get_bit(3)
    }

    /// Should be true if you want the segment to grow down rather than up
    fn set_direction_downward(&mut self, downward: bool) {
        self.0.set_bit(2, downward);
    }

    fn is_direction_downward(&self) -> bool {
        self.0.get_bit(2)
    }

    /// Readable bit for code selectors: Whether read access for this segment is allowed. Write access is never allowed for code segments.
    /// Writable bit for data selectors: Whether write access for this segment is allowed. Read access is always allowed for data segments.
    fn set_read_write(&mut self, read_write: bool) {
        self.0.set_bit(1, read_write);
    }

    fn is_read_write(&self) -> bool {
        self.0.get_bit(1)
    }

    fn is_accessed(&self) -> bool {
        self.0.get_bit(0)
    }
}

impl Debug for Flags {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let granularity = self.is_granularity();
        let size = self.is_size();
        let code_descriptor = self.is_code_descriptor();
        let present = self.is_present();
        let priviledge_level = self.get_priviledge_level();
        let descriptor_type = self.is_descriptor_type();
        let executable = self.is_executable();
        let downward = self.is_direction_downward();
        let read_write = self.is_read_write();
        let accessed = self.is_accessed();
        let mut debug = f.debug_struct("Access");
        debug.field("accessed", &accessed);
        debug.field("read_write", &read_write);
        debug.field("grows_downward", &downward);
        debug.field("executable", &executable);
        debug.field("descriptor_type", &descriptor_type);
        debug.field("priviledge_level", &priviledge_level);
        debug.field("present", &present);
        debug.field("code_descriptor", &code_descriptor);
        debug.field("size", &size);
        debug.field("granularity", &granularity);
        debug.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::size_of;

    /// Make sure the entry struct is getting correctly packed
    #[test_case]
    fn entry_struct_size() {
        assert_eq!(size_of::<Entry>(), 8);
    }

    /// Make sure the gdt struct is getting correctly packed
    #[test_case]
    fn gdt_struct_size() {
        assert_eq!(size_of::<GlobalDescriptorTable>(), 8 * 5);
    }
}
