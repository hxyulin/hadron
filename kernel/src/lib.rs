//! Hadron Kernel
#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::tests::test_runner)]
#![reexport_test_harness_main = "test_main"]

/// Boot shouldn't be accessible from the main kernel logic
pub(crate) mod boot;
pub mod serial;

#[cfg(test)]
mod tests {
    use core::panic::PanicInfo;

    pub fn test_runner(tests: &[&dyn Fn()]) {
        for test in tests {
            test();
        }
    }

    #[test_case]
    fn test_panic() {
        panic!("test panic");
    }

    #[panic_handler]
    fn panic(info: &PanicInfo) -> ! {
        let mut serial = unsafe { crate::serial::SerialPort::new(0x3F8) };
        use core::fmt::Write;
        writeln!(serial, "panic: {}", info).unwrap();
        loop {}
    }
}
