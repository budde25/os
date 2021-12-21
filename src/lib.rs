#![no_std]
#![cfg_attr(test, no_main)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![allow(dead_code)]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]

core::arch::global_asm!(include_str!("arch/x86_64/boot_32.s"));
core::arch::global_asm!(include_str!("arch/x86_64/boot_64.s"));

// export some common functionality
pub use address::PhysicalAddress;
pub use address::VirtualAddress;

extern crate alloc;

// pub so we can use them in integration tests
pub mod address;
pub mod arch;
pub mod consts;
pub mod disk;
pub mod interrupts;
pub mod io;
pub mod memory;
pub mod paging;
pub mod tables;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

/// Defines a run function
pub trait Testable {
    fn run(&self);
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        kprint!("{}...\t", core::any::type_name::<T>());
        self();
        kprintln!("[ok]");
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    kprintln!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &core::panic::PanicInfo) -> ! {
    kpanicprintln!("[failed]\n");
    kpanicprintln!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    interrupts::halt_loop();
}

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

#[cfg(test)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    test_panic_handler(info)
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use port::Port;
    let mut port = Port::new(0xf4);
    unsafe { port.write(exit_code as u32) };
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}
