#![no_std]
#![cfg_attr(test, no_main)]
#![test_runner(crate::common::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![allow(dead_code)]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(const_mut_refs)]

core::arch::global_asm!(include_str!("arch/x86_64/boot_32.s"));
core::arch::global_asm!(include_str!("arch/x86_64/boot_64.s"));

// export some common functionality
pub use address::PhysicalAddress;
pub use address::VirtualAddress;

extern crate alloc;

// pub so we can use them in integration tests
pub mod address;
pub mod arch;
pub mod common;
pub mod consts;
pub mod disk;
pub mod interrupts;
pub mod io;
pub mod memory;
pub mod paging;
pub mod tables;
pub mod task;

/// Entry point for `cargo test`
#[cfg(test)]
#[no_mangle]
pub extern "C" fn kmain() -> ! {
    // Map all of physical memory to addr + kernel offset
    paging::map_all_physical_memory();

    interrupts::init();
    test_main();

    interrupts::halt_loop();
}
