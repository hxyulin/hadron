//! Hadron Kernel
#![no_std]
#![no_main]
#![feature(custom_test_frameworks, abi_x86_interrupt)]
#![test_runner(crate::tests::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![allow(unexpected_cfgs)]

pub mod base;
/// Boot shouldn't be accessible from the main kernel logic
pub(crate) mod boot;
#[cfg(any(kernel_bootloader = "limine", feature = "never"))]
pub use boot::limine::limine_entry as kernel_entry;
use linked_list_allocator::LockedHeap;

#[cfg(not(any(kernel_bootloader = "limine", feature = "never")))]
compile_error!("No bootloader selected");

pub mod devices;

extern crate alloc;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

#[cfg(test)]
mod tests {
    use core::panic::PanicInfo;

    pub fn test_runner(tests: &[&dyn Fn()]) {
        for test in tests {
            test();
        }
    }

    #[panic_handler]
    fn panic(info: &PanicInfo) -> ! {
        let mut serial = unsafe { uart_16550::SerialPort::new(0x3F8) };
        serial.init();
        use core::fmt::Write;
        writeln!(serial, "panic: {}", info).unwrap();
        loop {}
    }
}
