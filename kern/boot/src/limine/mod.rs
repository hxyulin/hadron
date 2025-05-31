mod requests;

pub fn kernel_entry() -> ! {
    use core::fmt::Write;
    let mut serial = unsafe { uart_16550::SerialPort::new(0x3F8) };
    if !requests::BASE_REVISION.is_supported() {
        // Base Revision is not supported
    }
    serial.init();
    loop {}
}

