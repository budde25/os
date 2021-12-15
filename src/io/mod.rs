pub mod ioapic;
pub mod keyboard;
pub mod lapic;
pub mod pic;
pub mod uart;
pub mod vga;

use core::fmt::{Arguments, Write};
use lapic::Lapic;
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
    static ref PANIC_VGA: Mutex<Vga> = {
        let mut writer = Vga::new_panic();
        writer.clear_screen();
        Mutex::new(writer)
    };
    static ref PIC_1: Mutex<Pic> = {
        let pic = Pic::pic_1();
        Mutex::new(pic)
    };
    static ref PIC_2: Mutex<Pic> = {
        let pic = Pic::pic_2();
        Mutex::new(pic)
    };
    static ref LAPIC: Mutex<Lapic> = {
        let lapic = Lapic::default();
        Mutex::new(lapic)
    };
}

pub fn pic_init() {
    pic_remap();
    pic_mask();
    //pic_disable();
}

pub fn lapic_init() {
    LAPIC.lock().init();
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

/// mask all values
fn pic_mask() {
    PIC_1.lock().set_mask_all();
    PIC_2.lock().set_mask_all();
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
    ($val:expr,) => { $crate::kdbg!($val) };
    ($($val:expr),+ $(,)?) => {
        ($($crate::kdbg!($val)),+,)
    };
}

/// Print that writes to VGA buffer and Uart
#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => ($crate::io::_print(format_args!($($arg)*), false));
}

/// Panic use only, Print that writes to VGA buffer and Uart
#[macro_export]
macro_rules! kpanicprint {
    ($($arg:tt)*) => ($crate::io::_print(format_args!($($arg)*), true));
}

/// Print line that writes to VGA and Uart
#[macro_export]
macro_rules! kprintln {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::kprint!("{}\n", format_args!($($arg)*)));
}

/// Panic use only, Print line that writes to VGA and Uart
#[macro_export]
macro_rules! kpanicprintln {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::kpanicprint!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: Arguments, panic: bool) {
    _print_vga(args, panic);
    _print_uart(args, panic);
}

#[doc(hidden)]
pub fn _print_vga(args: Arguments, panic: bool) {
    use crate::interrupts::without_interrupts;

    without_interrupts(|| {
        if panic {
            PANIC_VGA.lock().write_fmt(args).unwrap();
        } else {
            VGA.lock().write_fmt(args).unwrap();
        }
    })
}

#[doc(hidden)]
pub fn _print_uart(args: Arguments, panic: bool) {
    use crate::interrupts::without_interrupts;

    without_interrupts(|| {
        if panic {
            unsafe { UART.force_unlock() };
            UART.lock().write_fmt(args).unwrap()
        } else {
            UART.lock().write_fmt(args).unwrap();
        }
    })
}
