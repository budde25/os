#![no_std]
#![cfg_attr(test, no_main)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![allow(dead_code)]
#![feature(custom_test_frameworks)]
#![feature(asm)]
#![feature(global_asm)]
#![feature(abi_x86_interrupt)]

global_asm!(include_str!("arch/x86_64/boot_32.s"));
global_asm!(include_str!("arch/x86_64/boot_64.s"));

// pub so we can use them in integration tests
pub mod address;
pub mod interrupts;
pub mod io;

use core::panic::PanicInfo;
use io::port::{Port, QemuExitCode};

/// Defines a run function
pub trait Testable {
    fn run(&self);
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        print!("{}...\t", core::any::type_name::<T>());
        self();
        println!("[ok]");
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    println!("[failed]\n");
    println!("Error: {}\n", info);
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
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}
