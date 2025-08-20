use core::arch::asm;

/// Enables Interrupts
#[inline]
pub unsafe fn enable() {
    unsafe { asm!("sti") }
}

/// Disables Interrupts
#[inline]
pub unsafe fn disable() {
    unsafe { asm!("cli") }
}
