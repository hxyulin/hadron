use core::panic::PanicInfo;

use spin::Mutex;

pub(crate) static ALT_PANIC_HANDLER: Mutex<Option<fn(&PanicInfo) -> !>> = Mutex::new(None);

#[panic_handler]
fn kernel_panic(info: &PanicInfo) -> ! {
    if let Some(handler) = *ALT_PANIC_HANDLER.lock() {
        handler(info);
    } else {
    }
    loop {}
}
