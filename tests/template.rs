#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(os::common::test_runner)]
#![reexport_test_harness_main = "test_main"]

#[no_mangle] // don't mangle the name of this function
pub extern "C" fn kmain() -> ! {
    test_main();

    loop {}
}
