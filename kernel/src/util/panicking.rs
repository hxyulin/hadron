use core::panic::PanicInfo;

use spin::Mutex;

use crate::kprintln;

static ALT_PANIC_HANDLER: Mutex<Option<fn(&PanicInfo) -> !>> = Mutex::new(None);

#[panic_handler]
fn kernel_panic(info: &PanicInfo) -> ! {
    if let Some(handler) = *ALT_PANIC_HANDLER.lock() {
        handler(info);
    } else {
        unsafe { crate::util::kprint::LOGGER.force_unlock() };
        kprintln!(Fatal, "panic: {}", info);
    }
    loop {}
}

pub fn set_alternate_panic_handler(panic: Option<fn(&PanicInfo) -> !>) {
    *ALT_PANIC_HANDLER.lock() = panic;
}
