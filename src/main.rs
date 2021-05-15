#![no_std]
#![no_main]
#![feature(
    panic_info_message,
    asm,
    global_asm,
    lang_items,
    custom_test_frameworks,
    llvm_asm
)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

use port::Port;

global_asm!(include_str!("arch/x86_64/boot_32.s"));
global_asm!(include_str!("arch/x86_64/boot_64.s"));

mod interrupts;
mod port;
mod uart;
mod vga;

#[no_mangle]
pub extern "C" fn kmain() -> ! {
    // Main should initialize all sub-systems and get
    // ready to start scheduling. The last thing this
    // should do is start the timer.
    interrupts::init();
    unsafe {
        uart::UART.lock().init();
    }

    // divide_by_zero();
    //println!("Hello World");

    #[cfg(test)]
    test_main();

    loop {}
}

fn divide_by_zero() {
    unsafe { llvm_asm!("mov dx, 0; div dx" ::: "ax", "dx" : "volatile", "intel") }
}

#[lang = "eh_personality"]
#[no_mangle]
extern "C" fn eh_personality() {
    loop {}
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    print!("Aborting: ");
    if let Some(p) = info.location() {
        println!(
            "line {}, file {}: {}",
            p.line(),
            p.file(),
            info.message().unwrap()
        );
    } else {
        println!("no information available.");
    }
    abort();
}

#[no_mangle]
extern "C" fn abort() -> ! {
    loop {}
}

pub fn exit_qemu(exit_code: u32) {
    unsafe {
        let port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
    exit_qemu(port::QemuExitCode::Success as u32)
}

#[test_case]
fn trivial_assertion() {
    print!("trivial assertion...");
    assert_eq!(1, 1);
    println!("[ok]");
}
