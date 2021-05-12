#![no_std]
#![no_main]
#![feature(
    panic_info_message,
    asm,
    global_asm,
    lang_items,
    custom_test_frameworks
)]
#![test_runner(crate::test_runner)]

global_asm!(include_str!("arch/x86_64/boot_32.s"));
global_asm!(include_str!("arch/x86_64/boot_64.s"));

mod vga;

#[no_mangle]
pub extern "C" fn kmain() {
    // Main should initialize all sub-systems and get
    // ready to start scheduling. The last thing this
    // should do is start the timer.

    println!("Hello, World");
    print!("line2");

    loop {}
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

#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
}
