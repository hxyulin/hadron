#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[unsafe(no_mangle)]
pub extern "C" fn kernel_entry() -> ! {
    if hadron_kernel::requests::BASE_REVISION.is_supported() {
        panic!("Limine is supported");
    }
    panic!("Hello, world!");
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let mut serial = unsafe { hadron_kernel::serial::SerialPort::new(0x3F8) };
    use core::fmt::Write;
    writeln!(serial, "panic: {}", info).unwrap();
    loop {}
}
