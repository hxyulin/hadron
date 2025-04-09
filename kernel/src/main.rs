#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[unsafe(no_mangle)]
extern "C" fn kernel_main() -> ! {
    panic!("Reached end of kernel");
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let mut serial = unsafe { hadron_kernel::serial::SerialPort::new(0x3F8) };
    use core::fmt::Write;
    writeln!(serial, "panic: {}", info).unwrap();
    loop {}
}
