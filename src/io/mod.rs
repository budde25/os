pub mod pic;
pub mod port;
pub mod uart;
pub mod vga;

use core::fmt::{Arguments, Write};
use lazy_static::lazy_static;
use pic::Pic;
use spin::Mutex;
use uart::Uart;
use vga::Vga;

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
    pic_disable();
}

pub fn uart_disable() {
    UART.lock().disable();
}

pub fn uart_enable() {
    UART.lock().enable();
}

fn pic_remap() {
    let offset = 32;
    PIC_1.lock().remap(offset);
    PIC_2.lock().remap(offset);
}

/// disables the PIC's
fn pic_disable() {
    PIC_1.lock().disable();
    PIC_2.lock().disable();
}

/// Print that writes to VGA buffer and Uart
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::io::_print(format_args!($($arg)*)));
}

/// Print line that writes to VGA and Uart
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

/// Print that writes to only Uart
#[macro_export]
macro_rules! uart {
    ($($arg:tt)*) => ($crate::io::_print_uart(format_args!($($arg)*)));
}

/// Print line that writes to only Uart
#[macro_export]
macro_rules! uartln {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::uart!("{}\n", format_args!($($arg)*)));
}

/// Print that writes to only VGA buffer
#[macro_export]
macro_rules! vga {
    ($($arg:tt)*) => ($crate::io::_print_vga(format_args!($($arg)*)));
}

/// Print line that writes to only VGA buffer
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
