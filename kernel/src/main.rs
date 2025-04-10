#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[unsafe(no_mangle)]
extern "C" fn kernel_main() -> ! {
    panic!("Reached end of kernel");
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    #[cfg(target_arch = "x86_64")]
    {
        use core::fmt::Write;
        let mut serial = unsafe { hadron_kernel::serial::SerialPort::new(0x3F8) };
        writeln!(serial, "panic: {}", info).unwrap();
    }
    loop {}
}
