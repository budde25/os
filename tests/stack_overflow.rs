#![no_std]
#![no_main]
#![feature(global_asm, custom_test_frameworks, abi_x86_interrupt)]
#![test_runner(os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use lazy_static::lazy_static;
use os::{
    exit_qemu,
    interrupts::{
        errors::ExceptionStackFrame, gdt, idt::InterruptDescriptorTable, DOUBLE_FAULT_IST_INDEX,
    },
    kprint, QemuExitCode,
};

static mut COUNTER: u64 = 0;

#[no_mangle] // don't mangle the name of this function
pub extern "C" fn kmain() -> ! {
    kprint!("stack_overflow::stack_overflow...\t");

    init();

    // trigger a stack overflow
    stack_overflow();

    panic!("Execution continued after stack overflow");
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    os::test_panic_handler(info)
}

#[allow(unconditional_recursion)]
fn stack_overflow() {
    unsafe {
        COUNTER += 1;
    }
    // makes a volatile write
    os::kprint!("{}", unsafe { COUNTER });
    stack_overflow(); // for each recursion, the return address is pushed
}

lazy_static! {
    static ref TEST_IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.double_fault.set_handler(test_double_fault_handler);
        idt.double_fault
            .options
            .set_stack_index(DOUBLE_FAULT_IST_INDEX);
        idt
    };
}

pub fn init() {
    use os::interrupts::GDT;
    GDT.0.load();
    TEST_IDT.load();

    unsafe {
        gdt::load_cs(GDT.1.kernel_code_segment);
        gdt::load_tss(GDT.1.tss_segment);
    }
}

extern "x86-interrupt" fn test_double_fault_handler(
    _stack_frame: ExceptionStackFrame,
    _error_code: u64,
) -> ! {
    os::kprintln!("[ok]");
    exit_qemu(QemuExitCode::Success);
    loop {}
}
