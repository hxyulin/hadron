#![no_main]
#![no_std]

pub mod logger;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    use log::Log;
    log::error!("Kernel Panic: {}", info);
    crate::logger::LOGGER.flush();
    loop {
        unsafe { core::arch::asm!("hlt", options(preserves_flags, nomem, nostack)) }
    }
}
