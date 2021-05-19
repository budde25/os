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

pub extern "x86-interrupt" fn divide_by_zero(stack_frame: ExceptionStackFrame) {
    println!("EXCEPTION: DIVIDE BY ZERO\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn page_fault(stack_frame: ExceptionStackFrame) {
    println!("EXCEPTION: PAGE FAULT\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn double_fault(
    stack_frame: ExceptionStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn breakpoint(stack_frame: ExceptionStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}
