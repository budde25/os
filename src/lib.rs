#![no_std]
#![cfg_attr(test, no_main)]
#![test_runner(crate::common::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![allow(dead_code)]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(const_mut_refs)]
#![feature(panic_info_message)]
#![feature(int_roundings)]

core::arch::global_asm!(include_str!("arch/x86_64/boot_32.s"));
core::arch::global_asm!(include_str!("arch/x86_64/boot_64.s"));
core::arch::global_asm!(include_str!("arch/x86_64/trampoline.s"));

extern crate alloc;

// pub so we can use them in integration tests
pub mod common;
pub mod consts;
pub mod disk;
pub mod interrupts;
pub mod io;
pub mod memory;
pub mod multiboot;
pub mod proc;
pub mod sections;
pub mod task;

/// Entry point for `cargo test`
#[cfg(test)]
#[no_mangle]
pub extern "C" fn kmain() -> ! {
    // Map all of physical memory to addr + kernel offset
    use sections::{Section, SECTIONS};
    x86_64::paging::map_all_physical_memory(SECTIONS[Section::PhysPageTable].start());

    interrupts::init();
    test_main();

    interrupts::halt_loop();
}
