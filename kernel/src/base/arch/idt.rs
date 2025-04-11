use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

lazy_static::lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt.double_fault.set_handler_fn(double_fault_handler);
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt
    };
}

pub fn init() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    panic!("Breakpoint hit at {:#x}", stack_frame.instruction_pointer);
}

extern "x86-interrupt" fn page_fault_handler(stack_frame: InterruptStackFrame, error_code: PageFaultErrorCode) {
    panic!(
        "Page fault at {:#x}, error code: {:#x}",
        stack_frame.instruction_pointer, error_code,
    );
}

extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, error_code: u64) -> ! {
    panic!(
        "Double fault at {:#x}, error code: {:#x}",
        stack_frame.instruction_pointer, error_code
    );
}
