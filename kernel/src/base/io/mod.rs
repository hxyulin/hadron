use spin::Mutex;

pub mod mmio;

/// Waits for the IO to finish.
///
/// # Safety
/// This function is unsafe because it interacts directly with the hardware.
#[inline]
pub unsafe fn io_wait() {
    unsafe {
        core::arch::asm!("out dx, al", in("dx") 0x80, in("al") 0i8);
    }
}

pub static MMIO: Mutex<mmio::KernelMmio> = Mutex::new(mmio::KernelMmio::new());
