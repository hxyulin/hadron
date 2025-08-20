use core::arch::asm;
use crate::arch::x86_64::core::{gdt::GDTDescriptor, idt::IDTRegister};

pub mod interrupts;

pub unsafe fn lgdt(gdtr: &GDTDescriptor) {
    unsafe {
        asm!("lgdt [{}]", in(reg) gdtr, options(readonly, nostack, preserves_flags));
    }
}

pub unsafe fn lidt(idtr: &IDTRegister) {
    unsafe {
        asm!("lidt [{}]", in(reg) idtr, options(readonly, nostack, preserves_flags));
    }
}
