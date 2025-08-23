use core::arch::asm;
use crate::arch::{registers::segmentation::SegmentSelector, x86_64::core::{gdt::GDTDescriptor, idt::IDTRegister}, VirtAddr};

pub mod interrupts;

#[inline]
pub unsafe fn lgdt(gdtr: &GDTDescriptor) {
    unsafe {
        asm!("lgdt [{}]", in(reg) gdtr, options(readonly, nostack, preserves_flags));
    }
}

#[inline]
pub unsafe fn lidt(idtr: &IDTRegister) {
    unsafe {
        asm!("lidt [{}]", in(reg) idtr, options(readonly, nostack, preserves_flags));
    }
}

#[inline]
pub unsafe fn load_tss(sel: SegmentSelector) {
    unsafe {
        asm!("ltr {0:x}", in(reg) sel.0, options(nostack, preserves_flags));
    }
}

#[inline]
pub unsafe fn invlpg(addr: VirtAddr) {
    unsafe {
        asm!(
            "invlpg [{}]",
            in(reg) addr.as_usize(),
            options(nostack, preserves_flags)
        );
    }
}
