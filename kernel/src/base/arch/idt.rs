use x86_64::{registers::control::Cr2, structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode}};

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
    log::info!("Breakpoint hit at {:#x}", stack_frame.instruction_pointer);
    log::info!("Stack trace: {:#x?}", stack_frame);
}

extern "x86-interrupt" fn page_fault_handler(stack_frame: InterruptStackFrame, error_code: PageFaultErrorCode) {
    // Page fault address is stored in CR2
    let fault_addr = Cr2::read().unwrap();
    log::error!(
        "Page fault at {:?}, error code: {:?}",
        fault_addr,
        error_code
    );
    log::error!("Stack trace: {:#x?}", stack_frame);
    panic!("Page fault");
}

extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, error_code: u64) -> ! {
    panic!(
        "Double fault at {:#x}, error code: {:#x}",
        stack_frame.instruction_pointer, error_code
    );
}
