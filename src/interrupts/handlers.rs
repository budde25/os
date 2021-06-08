use crate::println;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct ExceptionStackFrame {
    instruction_pointer: u64,
    code_segment: u64,
    cpu_flags: u64,
    stack_pointer: u64,
    stack_segment: u64,
}

/// 1
pub extern "x86-interrupt" fn divide_by_zero(stack_frame: ExceptionStackFrame) {
    println!("EXCEPTION: DIVIDE BY ZERO\n{:#?}", stack_frame);
}

/// 2
pub extern "x86-interrupt" fn debug(stack_frame: ExceptionStackFrame) {
    println!("EXCEPTION: DEBUG\n{:#?}", stack_frame);
}

/// 3
pub extern "x86-interrupt" fn non_maskable_interrupt(stack_frame: ExceptionStackFrame) {
    println!("EXCEPTION: NON MASKABLE INTERRUPT\n{:#?}", stack_frame);
}

/// 4
pub extern "x86-interrupt" fn breakpoint(stack_frame: ExceptionStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

/// 5
pub extern "x86-interrupt" fn overflow(stack_frame: ExceptionStackFrame) {
    println!("EXCEPTION: OVERFLOW\n{:#?}", stack_frame);
}

/// 6
pub extern "x86-interrupt" fn bound_range_exceeded(stack_frame: ExceptionStackFrame) {
    println!("EXCEPTION: BOUND RANGE EXCEEDED\n{:#?}", stack_frame);
}

/// 7
pub extern "x86-interrupt" fn invalid_opcode(stack_frame: ExceptionStackFrame) {
    println!("EXCEPTION: INVALID_OPCODE\n{:#?}", stack_frame);
}

/// 8
pub extern "x86-interrupt" fn device_not_available(stack_frame: ExceptionStackFrame) {
    println!("EXCEPTION: DEVICE NOT AVAILABLE\n{:#?}", stack_frame);
}

/// 9
pub extern "x86-interrupt" fn double_fault(
    stack_frame: ExceptionStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

/// 10
pub extern "x86-interrupt" fn invalid_tss(stack_frame: ExceptionStackFrame, error_code: u64) {
    println!(
        "EXCEPTION: INVALID_TSS\n{:#?}\nError Code:{}",
        stack_frame, error_code
    );
}

/// 11
pub extern "x86-interrupt" fn segment_not_present(
    stack_frame: ExceptionStackFrame,
    error_code: u64,
) {
    println!(
        "EXCEPTION: SEGMENT_NOT_PRESENT\n{:#?}\nError Code:{}",
        stack_frame, error_code
    );
}

/// 12
pub extern "x86-interrupt" fn stack_segment_fault(
    stack_frame: ExceptionStackFrame,
    error_code: u64,
) {
    println!(
        "EXCEPTION: STACK SEGMENT FAULT\n{:#?}\nError Code:{}",
        stack_frame, error_code
    );
}

/// 13
pub extern "x86-interrupt" fn general_protection_fault(
    stack_frame: ExceptionStackFrame,
    error_code: u64,
) {
    println!(
        "EXCEPTION: GENERAL PROTECTION FAULT\n{:#?}\nError Code:{}",
        stack_frame, error_code
    );
}

/// 14
pub extern "x86-interrupt" fn page_fault(stack_frame: ExceptionStackFrame, error_code: u64) {
    println!(
        "EXCEPTION: PAGE FAULT\n{:#?}\nError Code:{}",
        stack_frame, error_code
    );
}

/// 15
pub extern "x86-interrupt" fn x87_floating_point(stack_frame: ExceptionStackFrame) {
    println!("EXCEPTION: x87 FLOATING POINT\n{:#?}", stack_frame);
}

/// 16
pub extern "x86-interrupt" fn alignment_check(stack_frame: ExceptionStackFrame, error_code: u64) {
    println!(
        "EXCEPTION: x87 FLOATING POINT\n{:#?}\nError Code:{}",
        stack_frame, error_code
    );
}

/// 17
pub extern "x86-interrupt" fn machine_check(stack_frame: ExceptionStackFrame) -> ! {
    panic!("EXCEPTION: MACHINE CHECK\n{:#?}", stack_frame);
}

/// 18
pub extern "x86-interrupt" fn simd_floating_point(stack_frame: ExceptionStackFrame) {
    println!("EXCEPTION: SIMD FLOATING POINT\n{:#?}", stack_frame);
}

/// 19
pub extern "x86-interrupt" fn virtualization(stack_frame: ExceptionStackFrame) {
    println!("EXCEPTION: VIRTUALIZATION\n{:#?}", stack_frame);
}

/// 20
pub extern "x86-interrupt" fn security_exception(
    stack_frame: ExceptionStackFrame,
    error_code: u64,
) {
    println!(
        "EXCEPTION: VIRTUALIZATION\n{:#?}\nError Code:{}",
        stack_frame, error_code
    );
}
