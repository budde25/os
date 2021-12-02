#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(asm)]
#![feature(global_asm)]
#![feature(lang_items)]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![allow(dead_code)]

global_asm!(include_str!("arch/x86_64/boot_32.s"));
global_asm!(include_str!("arch/x86_64/boot_64.s"));

extern crate alloc;

mod address;
mod allocator;
mod arch;
mod interrupts;
mod io;
mod paging;
mod tables;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

/// Kernel entry point
#[no_mangle]
pub extern "C" fn kmain() -> ! {
    // Main should initialize all sub-systems and get
    // ready to start scheduling. The last thing this
    // should do is start the timer.
    interrupts::init();
    // Remap and disable the pic
    io::pic_init();

    // enable heap, requires page table
    //allocator::init();

    // enable interrupts
    kernel_println!("Before Interrupt");
    interrupts::enable_interrupts();

    #[cfg(test)]
    test_main();

    kernel_println!("Hello World");

    interrupts::halt_loop();
}

#[lang = "eh_personality"]
#[no_mangle]
extern "C" fn eh_personality() {
    interrupts::halt_loop();
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    kernel_print!("Aborting: ");
    if let Some(p) = info.location() {
        kernel_println!(
            "line {}, file {}: {}",
            p.line(),
            p.file(),
            info.message().unwrap()
        );
    } else {
        kernel_println!("no information available.");
    }
    abort();
}

#[no_mangle]
#[cfg(not(test))]
extern "C" fn abort() -> ! {
    interrupts::halt_loop();
}

#[no_mangle]
#[cfg(test)]
extern "C" fn abort() -> ! {
    exit_qemu(QemuExitCode::Failed as u32);
    interrupts::halt_loop();
}

pub fn exit_qemu(exit_code: u32) {
    use port::Port;
    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

#[cfg(test)]
fn test_runner(tests: &[&dyn Testable]) {
    kernel_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success as u32)
}

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

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}
