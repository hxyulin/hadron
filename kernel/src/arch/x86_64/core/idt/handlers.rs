use crate::util::machine_state::MachineState;

use super::InterruptStackFrame;

pub(super) extern "x86-interrupt" fn divide_by_zero(stack_frame: InterruptStackFrame) {
    panic!("FAULT: DIVIDE BY ZERO\nstack frame: {:#?}", stack_frame);
}

pub(super) extern "x86-interrupt" fn debug(stack_frame: InterruptStackFrame) {
    panic!("TRAP: DEBUG\nstack frame: {:#?}", stack_frame);
}

pub(super) extern "x86-interrupt" fn breakpoint(stack_frame: InterruptStackFrame) {
    panic!("TRAP: BREAKPOINT\nstack frame: {:#?}", stack_frame);
}

pub(super) extern "x86-interrupt" fn double_fault(stack_frame: InterruptStackFrame, err_code: u64) -> ! {
    let state = MachineState::from_stack_frame(&stack_frame);
    panic!("ABORT: DOUBLE_FAULT\nerror_code: {}\nstack frame: {:#?}\nstate: {}", err_code, stack_frame, state);
}

pub(super) extern "x86-interrupt" fn page_fault(stack_frame: InterruptStackFrame, err_code: u64) {
    panic!("TRAP: PAGE_FAULT\nstack frame: {:#?}\nerr_code = {}", stack_frame, err_code);
}
