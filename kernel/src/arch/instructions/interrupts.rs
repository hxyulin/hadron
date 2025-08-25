use core::arch::asm;

/// Enables Interrupts
#[inline]
pub unsafe fn enable() {
    if cfg!(target_arch = "x86_64") {
        unsafe { asm!("sti") }
    } else {
        unimplemented!();
    }
}

/// Disables Interrupts
#[inline]
pub unsafe fn disable() {
    if cfg!(target_arch = "x86_64") {
        unsafe { asm!("cli") }
    } else {
        unimplemented!();
    }
}
