use sections::Sections;

// export up
pub use phys::PhysicalAddress;
use spin::Lazy;
pub use virt::VirtualAddress;

mod phys;
pub mod sections;
mod virt;

pub const fn align_down(addr: u64, align: u64) -> u64 {
    assert!(align.is_power_of_two(), "`align` must be a power of two");
    addr & !(align - 1)
}

pub const fn align_up(addr: u64, align: u64) -> u64 {
    assert!(align.is_power_of_two(), "`align` must be a power of two");
    let align_mask = align - 1;
    if addr & align_mask == 0 {
        addr // already aligned
    } else {
        (addr | align_mask) + 1
    }
}

pub static SECTIONS: Lazy<Sections> = Lazy::new(Sections::new);
