use crate::interrupts::halt_loop;
use crate::{kprint, kprintln};

pub const GREEN: &str = "\x1b[0;32m";
pub const NC: &str = "\x1b[0m";
pub const RED: &str = "\x1b[0;31m";

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

pub trait Testable {
    fn run(&self);
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        kprint!("{:60}", core::any::type_name::<T>());
        self();
        kprintln!("{GREEN}[Ok]{NC}");
    }
}

#[no_mangle]
pub extern "C" fn abort() -> ! {
    #[cfg(not(test))]
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

pub fn test_runner(tests: &[&dyn Testable]) {
    kprintln!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    kprintln!("{GREEN}{} tests passed!{NC}", tests.len());
    exit_qemu(true)
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    let fail = "No message available";

    if let Some(p) = info.location() {
        let message = match info.message() {
            Some(message) => message.as_str().unwrap_or(fail),
            None => fail,
        };
        kprintln!("Panic: [{}:{}] {}", p.file(), p.line(), message);
    } else {
        kprintln!("Panic: No information available");
    }
    #[cfg(test)]
    {
        exit_qemu(false);
        halt_loop();
    }

    abort()
}
