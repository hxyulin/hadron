use x86_64::{
    registers::control::Cr2,
    structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode},
};

use crate::util::machine_state::MachineState;

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

#[inline(always)]
fn dump_machine_state(stack_frame: &InterruptStackFrame) {
    let machine_state = MachineState::from_stack_frame(stack_frame);
    log::info!("Machine state: \n{}", machine_state);
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    log::info!("Breakpoint hit at {:#x}", stack_frame.instruction_pointer);
    log::info!("Stack trace: {:#x?}", stack_frame);
    dump_machine_state(&stack_frame);
}

extern "x86-interrupt" fn page_fault_handler(stack_frame: InterruptStackFrame, error_code: PageFaultErrorCode) {
    // Page fault address is stored in CR2
    let fault_addr = Cr2::read().unwrap();
    log::error!("Page fault at {:?}, error code: {:?}", fault_addr, error_code);
    log::error!("Stack trace: {:#x?}", stack_frame);
    dump_machine_state(&stack_frame);
    panic!("Page fault at {:?}, error code: {:?}", fault_addr, error_code);
}

extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, error_code: u64) -> ! {
    dump_machine_state(&stack_frame);
    panic!(
        "Double fault at {:#x}, error code: {:#x}",
        stack_frame.instruction_pointer, error_code
    );
}
