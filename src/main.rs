#![no_std]
#![no_main]
#![feature(
    panic_info_message,
    asm,
    global_asm,
    lang_items,
    custom_test_frameworks,
    llvm_asm,
    abi_x86_interrupt
)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![allow(dead_code)]

global_asm!(include_str!("arch/x86_64/boot_32.s"));
global_asm!(include_str!("arch/x86_64/boot_64.s"));

mod address;
mod interrupts;
mod io;

/// Kernel entry point
#[no_mangle]
pub extern "C" fn kmain() -> ! {
    // Main should initialize all sub-systems and get
    // ready to start scheduling. The last thing this
    // should do is start the timer.
    interrupts::init();
    // Remap and disable the pic
    io::pic_init();

    unsafe {
        // interrupts::enable_interrupts();
    }

    #[cfg(test)]
    test_main();

    println!("Hello World");

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
#[cfg(not(test))]
extern "C" fn abort() -> ! {
    loop {}
}

#[no_mangle]
#[cfg(test)]
extern "C" fn abort() -> ! {
    use io::port::QemuExitCode;
    exit_qemu(QemuExitCode::Failed as u32);
    loop {}
}

pub fn exit_qemu(exit_code: u32) {
    use io::port::Port;
    unsafe {
        let port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

#[cfg(test)]
fn test_runner(tests: &[&dyn Testable]) {
    use io::port::QemuExitCode;
    println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success as u32)
}

pub trait Testable {
    fn run(&self) -> ();
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

mod tests {
    #[test_case]
    fn trivial_assertion() {
        assert_eq!(1, 1);
    }
}
