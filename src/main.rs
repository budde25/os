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
core::arch::global_asm!(include_str!("arch/x86_64/mp_boot.s"));

// export some common functionality
pub use address::PhysicalAddress;
pub use address::VirtualAddress;

extern crate alloc;

mod address;
mod common;
mod consts;
mod disk;
mod interrupts;
mod io;
mod memory;
mod paging;
mod proc;
mod registers;
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

    kprintln!("Current time: {}", io::current_time());

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
    use paging::allocator::Allocator;
    use proc::cpu::Cpu;
    use tables::MADT_TABLE;

    let _aps_running = 0;

    let num_cores = MADT_TABLE.num_cores();
    let lapic_ids = MADT_TABLE.apic_ids();
    let code = PhysicalAddress::new(0x7000);

    // move the code
    extern "C" {
        static __mp_boot_start: usize;
        static __mp_boot_end: usize;
    }
    let mp_boot_start = unsafe { &__mp_boot_start as *const _ as *const u8 };
    let mp_boot_size =
        unsafe { &__mp_boot_end as *const _ as usize - &__mp_boot_start as *const _ as usize };

    let src = PhysicalAddress::new(mp_boot_start as u64);

    unsafe { mem_copy(code.as_mut_ptr::<u8>(), src.as_ptr::<u8>(), mp_boot_size) };

    for i in 0..num_cores {
        let _ = Cpu::current_cpu();

        let lapic_id = lapic_ids[i as usize].unwrap();
        if lapic_id == 0 {
            continue;
        }

        let stack = paging::MAPPER.lock().allocate_frame().unwrap();
        let stack_addr = u64::from(stack.address());
        let code_ptr = code.as_mut_ptr::<u64>();
        unsafe { code_ptr.sub(1).write_volatile(stack_addr + 4096) };
        unsafe {
            code_ptr
                .sub(2)
                .write_volatile(mp_enter as *const u64 as u64)
        };

        unsafe { (*io::LAPIC.as_mut_ptr()).start_ap(lapic_id, code) };
    }
}

unsafe fn mem_copy(dst: *mut u8, src: *const u8, len: usize) {
    for i in 0..len {
        dst.add(i).write_volatile(src.add(i).read_volatile())
    }
}

#[lang = "eh_personality"]
#[no_mangle]
extern "C" fn eh_personality() {
    interrupts::halt_loop();
}

#[no_mangle]
extern "C" fn mp_enter() {
    kdbg!("here");
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
