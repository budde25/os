#![no_std]
#![no_main]
#![feature(panic_info_message,asm, global_asm, lang_items)]

global_asm!(include_str!("arch/x86_64/boot_1.s"));
global_asm!(include_str!("arch/x86_64/boot_2.s"));
global_asm!(include_str!("arch/x86_64/boot_3.s"));

#[no_mangle]
pub extern "C" fn kmain() {
	// Main should initialize all sub-systems and get
	// ready to start scheduling. The last thing this
	// should do is start the timer.

	// loop {}

    let hello = b"Hello World!";
    let color_byte = 0x1f; // white foreground, blue background

    let mut hello_colored = [color_byte; 24];
    for (i, char_byte) in hello.into_iter().enumerate() {
        hello_colored[i*2] = *char_byte;
    }

    // write `Hello World!` to the center of the VGA text buffer
    let buffer_ptr = (0xb8000 + 1988) as *mut _;
    unsafe { *buffer_ptr = hello_colored };

    loop{}
}

#[macro_export]
macro_rules! print
{
	($($args:tt)+) => ({

	});
}
#[macro_export]
macro_rules! println
{
	() => ({
		print!("\r\n")
	});
	($fmt:expr) => ({
		print!(concat!($fmt, "\r\n"))
	});
	($fmt:expr, $($args:tt)+) => ({
		print!(concat!($fmt, "\r\n"), $($args)+)
	});
}

#[lang = "eh_personality"]
#[no_mangle]
extern "C" fn eh_personality() {
	loop { }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
	print!("Aborting: ");
	if let Some(_p) = info.location() {
		println!(
					"line {}, file {}: {}",
					p.line(),
					p.file(),
					info.message().unwrap()
		);
	}
	else {
		println!("no information available.");
	}
	abort();
}

#[no_mangle]
extern "C"
fn abort() -> ! {
	loop {
	}
}		
