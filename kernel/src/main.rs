//! A stub kernel entry point for the kernel
//! We need to do this because we need to make it a lib for integration tests

#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[unsafe(no_mangle)]
extern "C" fn kernel_entry() -> ! {
    hadron_kernel::kernel_entry()
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    use core::fmt::Write;
    let mut serial = unsafe { uart_16550::SerialPort::new(0x3F8) };
    writeln!(serial, "panic: {}", info).unwrap();
    loop {
        x86_64::instructions::hlt();
    }
}
