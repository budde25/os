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
mod arch;
mod consts;
mod disk;
mod interrupts;
mod io;
mod memory;
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

    // Map all of physical memory to addr + kernel offset,
    // we should start with this to avoid errors with physical addrs
    paging::map_all_physical_memory();

    // log that we are starting
    if let Some(name) = tables::MULTIBOOT.boot_loader_name {
        kprintln!("Booting from: {}", name.string())
    }

    kprintln!("Number of cores: {}", tables::MADT_TABLE.num_cores());

    interrupts::init();
    // TODO: enable the lapic
    io::lapic_init();
    // Remap and disable the pic
    io::pic_init();
    // enable the heap
    memory::heap::init();
    // enable interrupts
    interrupts::enable_interrupts();

    kprintln!("Current time: {}", io::CMOS.lock().time());

    kprintln!("Hello World");

    interrupts::halt_loop();
}

#[lang = "eh_personality"]
#[no_mangle]
extern "C" fn eh_personality() {
    interrupts::halt_loop();
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    kpanicprint!("Aborting: ");
    if let Some(p) = info.location() {
        kpanicprintln!(
            "line {}, file {}: {}",
            p.line(),
            p.file(),
            info.message().unwrap()
        );
    } else {
        kpanicprintln!("no information available.");
    }
    abort()
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
    kprintln!("Running {} tests", tests.len());
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
        kprint!("{}...\t", core::any::type_name::<T>());
        self();
        kprintln!("[ok]");
    }
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}
