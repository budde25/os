use crate::interrupts::halt_loop;
use crate::{kprint, kprintln};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    test_panic_handler(info)
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

#[no_mangle]
#[cfg(not(test))]
pub extern "C" fn abort() -> ! {
    halt_loop();
}

#[no_mangle]
#[cfg(test)]
pub extern "C" fn abort() -> ! {
    exit_qemu(false);
    halt_loop();
}

pub fn exit_qemu(success: bool) {
    use port::Port;

    let exit_code = if success {
        QemuExitCode::Success as u32
    } else {
        QemuExitCode::Failed as u32
    };

    let mut port = Port::new(0xf4);
    unsafe { port.write(exit_code) };
}

#[cfg(not(test))]
pub fn test_runner(tests: &[&dyn Testable]) {
    kprintln!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(true)
}

#[cfg(test)]
pub fn test_runner(tests: &[&dyn Testable]) {
    kprintln!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(true)
}

pub fn test_panic_handler(info: &core::panic::PanicInfo) -> ! {
    crate::kpanicprintln!("[failed]\n");
    crate::kpanicprintln!("Error: {}\n", info);
    exit_qemu(false);
    halt_loop();
}
