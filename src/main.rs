#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(lang_items)]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(const_mut_refs)]
#![feature(int_roundings)]
#![test_runner(crate::common::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![allow(dead_code)]

core::arch::global_asm!(include_str!("arch/x86_64/boot_32.s"));
core::arch::global_asm!(include_str!("arch/x86_64/boot_64.s"));
core::arch::global_asm!(include_str!("arch/x86_64/trampoline.s"));

extern crate alloc;

mod common;
mod consts;
mod disk;
mod interrupts;
mod io;
mod memory;
mod multiboot;
mod paging;
mod proc;
mod sections;
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
    use sections::{Section, SECTIONS};
    paging::map_all_physical_memory(SECTIONS[Section::PhysPageTable].start());

    // Load GDT and IDT
    interrupts::init();

    // log that we are starting
    if let Some(name) = multiboot::MULTIBOOT_INFO.boot_loader_name() {
        kprintln!("Booting from: {}", name.string())
    }

    kprintln!("Number of cores: {}", multiboot::MADT_TABLE.num_cores());

    // enable the lapic
    io::lapic_init();
    // Remap and disable the pic
    io::pic_init();

    io::ioapic_init();
    // enable the heap
    memory::heap::init();
    // enable ide driver
    disk::ide_init();

    disk::ide_test();
    kprintln!("Current time: {}", io::current_time());

    // start additional processors
    // TODO: finish ap functionality
    // proc::ap_startup();

    // enable interrupts
    interrupts::enable_interrupts();

    use task::executor::Executor;
    use task::Task;

    kprintln!("Hello World");

    let mut executor = Executor::new();
    executor.spawn(Task::new(io::keyboard::print_keypresses()));
    //executor.spawn(Task::new(disk::ide::page_handler()));
    //executor.spawn(Task::new(disk::ide_test()));
    executor.run();
}

#[lang = "eh_personality"]
#[no_mangle]
extern "C" fn eh_personality() {
    interrupts::halt_loop();
}
