pub mod port;
pub mod uart;
pub mod vga;

use core::fmt::{Arguments, Write};
use lazy_static::lazy_static;
use spin::Mutex;
use uart::Uart;
use vga::Writer;

lazy_static! {
    static ref VGA: Mutex<Writer> = {
        let mut writer = Writer::default();
        writer.clear_screen();
        Mutex::new(writer)
    };
    static ref UART: Mutex<Uart> = {
        let mut uart = Uart::default();
        unsafe { uart.init() };
        Mutex::new(uart)
    };
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::io::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! uart {
    ($($arg:tt)*) => ($crate::io::_print_uart(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! uartln {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::uart!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! vga {
    ($($arg:tt)*) => ($crate::io::_print_vga(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! vgaln {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::vga!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: Arguments) {
    _print_vga(args);
    _print_uart(args);
}

#[doc(hidden)]
pub fn _print_vga(args: Arguments) {
    VGA.lock().write_fmt(args).unwrap();
}

#[doc(hidden)]
pub fn _print_uart(args: Arguments) {
    UART.lock().write_fmt(args).unwrap();
}
