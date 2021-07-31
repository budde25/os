#![no_std]
#![cfg_attr(test, no_main)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![allow(dead_code)]
#![feature(custom_test_frameworks)]
#![feature(asm)]
#![feature(global_asm)]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]

global_asm!(include_str!("arch/x86_64/boot_32.s"));
global_asm!(include_str!("arch/x86_64/boot_64.s"));

extern crate alloc;

// pub so we can use them in integration tests
pub mod address;
pub mod allocator;
pub mod arch;
pub mod interrupts;
pub mod io;
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
        kernel_print!("{}...\t", core::any::type_name::<T>());
        self();
        kernel_println!("[ok]");
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    kernel_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &core::panic::PanicInfo) -> ! {
    kernel_println!("[failed]\n");
    kernel_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    interrupts::halt_loop();
}

/// Entry point for `cargo test`
#[cfg(test)]
#[no_mangle]
pub extern "C" fn kmain() -> ! {
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
