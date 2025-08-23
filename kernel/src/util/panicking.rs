use core::panic::PanicInfo;

use spin::Mutex;

static ALT_PANIC_HANDLER: Mutex<Option<fn(&PanicInfo) -> !>> = Mutex::new(None);

#[panic_handler]
fn kernel_panic(info: &PanicInfo) -> ! {
    if let Some(handler) = *ALT_PANIC_HANDLER.lock() {
        handler(info);
    } else {
    }
    loop {}
}

pub fn set_alternate_panic_handler(panic: Option<fn(&PanicInfo) -> !>) {
    *ALT_PANIC_HANDLER.lock() = panic;
}
