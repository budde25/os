pub mod ioapic;
pub mod keyboard;
pub mod lapic;
pub mod pic;
pub mod uart;
pub mod vga;

use core::fmt::{Arguments, Write};
use lazy_static::lazy_static;
use pic::Pic;
use spin::Mutex;
use uart::Uart;
use vga::Vga;

pub const IRQ_0: u8 = 32;

lazy_static! {
    static ref VGA: Mutex<Vga> = {
        let mut writer = Vga::default();
        writer.clear_screen();
        Mutex::new(writer)
    };
    static ref UART: Mutex<Uart> = {
        let mut uart = Uart::default();
        unsafe { uart.init() };
        Mutex::new(uart)
    };
    static ref PIC_1: Mutex<Pic> = {
        let pic = Pic::pic_1();
        Mutex::new(pic)
    };
    static ref PIC_2: Mutex<Pic> = {
        let pic = Pic::pic_2();
        Mutex::new(pic)
    };
}

pub fn pic_init() {
    pic_remap();
    //pic_disable();
}

pub fn pic_eoi(index: usize) {
    // we always send it to the master but the slave too if it came from there
    if index >= 8 {
        PIC_2.lock().end_of_interrupt();
    }

    PIC_1.lock().end_of_interrupt();
}

pub fn uart_disable() {
    UART.lock().disable();
}

pub fn uart_enable() {
    UART.lock().enable();
}

fn pic_remap() {
    PIC_1.lock().remap();
    PIC_2.lock().remap();
}

/// disables the PIC's
fn pic_disable() {
    PIC_1.lock().disable();
    PIC_2.lock().disable();
}

#[macro_export]
macro_rules! kdbg {
    () => {
        $crate::kprintln!("[{}:{}]", file!(), line!());
    };
    ($val:expr) => {
        // Use of `match` here is intentional because it affects the lifetimes
        // of temporaries - https://stackoverflow.com/a/48732525/1063961
        match $val {
            tmp => {
                $crate::kprintln!("[{}:{}] {} = {:#?}",
                    file!(), line!(), stringify!($val), &tmp);
                tmp
            }
        }
    };
    // Trailing comma with single argument is ignored
    ($val:expr,) => { $crate::kernel_dbg!($val) };
    ($($val:expr),+ $(,)?) => {
        ($($crate::kdbg!($val)),+,)
    };
}

/// Print that writes to VGA buffer and Uart
#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => ($crate::io::_print(format_args!($($arg)*)));
}

/// Print line that writes to VGA and Uart
#[macro_export]
macro_rules! kprintln {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::kprint!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: Arguments) {
    _print_vga(args);
    _print_uart(args);
}

#[doc(hidden)]
pub fn _print_vga(args: Arguments) {
    use crate::interrupts::without_interrupts;

    without_interrupts(|| {
        VGA.lock().write_fmt(args).unwrap();
    })
}

#[doc(hidden)]
pub fn _print_uart(args: Arguments) {
    use crate::interrupts::without_interrupts;

    without_interrupts(|| {
        UART.lock().write_fmt(args).unwrap();
    })
}
