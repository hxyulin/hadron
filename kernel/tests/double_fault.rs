#![no_main]
#![no_std]
#![feature(abi_x86_interrupt)]

use hadron_kernel::{
    arch::x86_64::core::{
        gdt::{self, Selectors},
        idt::{InterruptDescriptorTable, InterruptStackFrame},
    },
    util::panicking::set_alternate_panic_handler,
};
use hadron_test::{ExitCode, exit_qemu, println};

lazy_static::lazy_static! {
    static ref IDT: InterruptDescriptorTable<'static> = {
        let mut idt = InterruptDescriptorTable::new();
        unsafe {
        idt.double_fault.set_handler_fn(double_fault)
            .set_stack_index(Selectors::DOUBLE_FAULT_IST_INDEX as u16);
        }
        idt
    };
}

#[unsafe(no_mangle)]
extern "C" fn kernel_entry() -> ! {
    set_alternate_panic_handler(Some(hadron_test::panic));

    unsafe {
        gdt::init();
    }
    IDT.load();
    println!("testing double fault (corrupting stack)...");
    println!("if this is the last message then the pass failed");
    #[allow(unconditional_recursion)]
    #[inline(never)]
    fn recurse() {
        recurse();
    }
    recurse();

    exit_qemu(ExitCode::Failed);
    loop {}
}

extern "x86-interrupt" fn double_fault(_stack_frame: InterruptStackFrame, _err_code: u64) -> ! {
    println!("double fault handler triggered (test passed)");
    exit_qemu(ExitCode::Success);
    loop {}
}
