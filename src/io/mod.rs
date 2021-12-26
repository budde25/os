pub mod cmos;
pub mod console;
pub mod ioapic;
pub mod keyboard;
pub mod lapic;
pub mod pic;
pub mod uart;
pub mod vga;

use cmos::Cmos;
use core::fmt::{Arguments, Write};
use ioapic::IOApicRef;
use lapic::Lapic;
use spin::{Lazy, Mutex};
use uart::Uart;
use vga::Vga;

use self::pic::Pics;

static VGA: Lazy<Mutex<Vga>> = Lazy::new(|| {
    let mut writer = Vga::default();
    writer.clear_screen();
    Mutex::new(writer)
});

static UART: Lazy<Mutex<Uart>> = Lazy::new(|| {
    let mut uart = Uart::default();
    unsafe { uart.init() };
    Mutex::new(uart)
});

static PANIC_VGA: Lazy<Mutex<Vga>> = Lazy::new(|| {
    let mut writer = Vga::new_panic();
    writer.clear_screen();
    Mutex::new(writer)
});

static PICS: Lazy<Mutex<Pics>> = Lazy::new(|| {
    let pics = Pics::default();
    Mutex::new(pics)
});

static LAPIC: Lazy<Mutex<Lapic>> = Lazy::new(|| {
    let lapic = Lapic::default();
    Mutex::new(lapic)
});

pub static CMOS: Lazy<Mutex<Cmos>> = Lazy::new(|| {
    let cmos = Cmos::default();
    Mutex::new(cmos)
});

pub static IO_APIC: Lazy<Mutex<IOApicRef>> = Lazy::new(|| {
    let ioapic = IOApicRef::default();
    Mutex::new(ioapic)
});

pub fn pic_init() {
    let mut pics = PICS.lock();
    pics.remap();
    pics.mask_all();
    pics.disable();
    crate::kprintln!("PIC's have been remaped, masked, and disabled");
}

pub fn lapic_init() {
    use crate::kprintln;

    LAPIC.lock().init();
    let status = LAPIC.lock().error_status();
    if status.is_empty() {
        kprintln!("LAPIC has been initialized");
    } else {
        kprintln!(
            "LAPIC initialization has failed with error(s): {:#?}",
            status
        );
    }
}

pub fn lapic_eoi() {
    LAPIC.lock().end_of_interrupt();
}

pub fn uart_disable() {
    UART.lock().disable();
}

pub fn uart_enable() {
    UART.lock().enable();
}

pub fn ioapic_init() {
    use crate::consts::IRQ;
    IO_APIC.lock().init();
    IO_APIC.lock().enable(IRQ::Keyboard, 0);

    crate::kprintln!("IOAPIC has been initialized");
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
