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
mod proc;
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

    // Load GDT and IDT
    interrupts::init();

    // log that we are starting
    if let Some(name) = tables::MULTIBOOT.boot_loader_name {
        kprintln!("Booting from: {}", name.string())
    }

    kprintln!("Number of cores: {}", tables::MADT_TABLE.num_cores());

    // enable the lapic
    io::lapic_init();
    // Remap and disable the pic
    io::pic_init();

    io::ioapic_init();
    // enable the heap
    memory::heap::init();
    // enable ide driver
    disk::ide_init();

    kprintln!("Current time: {}", io::CMOS.lock().time());

    // start addional processors
    ap_startup();

    // disk::ide_test();
    // enable interrupts
    interrupts::enable_interrupts();

    use task::executor::Executor;
    use task::Task;

    kprintln!("Hello World");

    let mut executor = Executor::new();
    executor.spawn(Task::new(io::keyboard::print_keypresses()));
    executor.run();
}

fn ap_startup() {
    use proc::cpu::Cpu;
    use tables::MADT_TABLE;

    let _aps_running = 0;

    let num_cores = MADT_TABLE.num_cores();
    let lapic_ids = MADT_TABLE.apic_ids();

    for i in 0..num_cores {
        let a = Cpu::current_cpu();

        let lapic_id = lapic_ids[i as usize].unwrap();
        kdbg!(lapic_id);
    }
}

#[lang = "eh_personality"]
#[no_mangle]
extern "C" fn eh_personality() {
    interrupts::halt_loop();
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    use crate::io::colors::{NC, RED};
    crate::io::kpanicprint!("{RED}Aborting: ");
    if let Some(p) = info.location() {
        crate::io::kpanicprintln!(
            "[{}:{}] {}{NC}",
            p.file(),
            p.line(),
            info.message().unwrap()
        );
    } else {
        crate::io::kpanicprintln!("no information available.");
    }
    common::abort()
}
