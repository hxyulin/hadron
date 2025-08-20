use super::InterruptStackFrame;

pub(super) extern "x86-interrupt" fn divide_by_zero(stack_frame: InterruptStackFrame) {
    panic!("DIVIDE BY ZERO\nstack frame: {:#?}", stack_frame);
}

pub(super) extern "x86-interrupt" fn breakpoint(stack_frame: InterruptStackFrame) {
    panic!("BREAKPOINT\nstack frame: {:#?}", stack_frame);
}
