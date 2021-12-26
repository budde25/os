#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(lang_items)]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(const_mut_refs)]
#![test_runner(crate::common::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![allow(dead_code)]

core::arch::global_asm!(include_str!("arch/x86_64/boot_32.s"));
core::arch::global_asm!(include_str!("arch/x86_64/boot_64.s"));

// export some common functionality
pub use address::PhysicalAddress;
pub use address::VirtualAddress;

extern crate alloc;

mod address;
mod arch;
mod common;
mod consts;
mod disk;
mod interrupts;
mod io;
mod memory;
mod paging;
mod tables;
mod task;

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

    // Load GDT and IDT
    interrupts::init();

    // enable the lapic
    io::lapic_init();
    // Remap and disable the pic
    io::pic_init();

    io::ioapic_init();
    // enable the heap
    memory::heap::init();
    // enable ide driver
    disk::ide_init();
    // enable interrupts
    interrupts::enable_interrupts();

    kprintln!("Current time: {}", io::CMOS.lock().time());

    use task::executor::Executor;
    use task::Task;

    kprintln!("Hello World");

    let mut executor = Executor::new();
    executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(io::keyboard::print_keypresses()));
    executor.run();
}

async fn async_number() -> u32 {
    42
}

async fn example_task() {
    let number = async_number().await;
    kprintln!("async number: {}", number);
}

#[lang = "eh_personality"]
#[no_mangle]
extern "C" fn eh_personality() {
    interrupts::halt_loop();
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    crate::kpanicprint!("Aborting: ");
    if let Some(p) = info.location() {
        crate::kpanicprintln!("[{}:{}] {}", p.file(), p.line(), info.message().unwrap());
    } else {
        crate::kpanicprintln!("no information available.");
    }
    common::abort()
}
