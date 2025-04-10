//! Hadron Kernel
#![no_std]
#![no_main]
#![feature(custom_test_frameworks, abi_x86_interrupt)]
#![test_runner(crate::tests::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![allow(unexpected_cfgs)]

use base::info::kernel_info;
use linked_list_allocator::LockedHeap;

pub mod base;
/// Boot shouldn't be accessible from the main kernel logic
pub(crate) mod boot;
pub mod devices;

#[cfg(any(kernel_bootloader = "limine", feature = "never"))]
pub use boot::limine::limine_entry as kernel_entry;
use x86_64::PhysAddr;

#[cfg(not(any(kernel_bootloader = "limine", feature = "never")))]
compile_error!("No bootloader selected");

extern crate alloc;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct KernelParams {
    pub rsdp: PhysAddr,
}

/// The main kernel 'entry point'
/// It is sort of an intermediate stage, which is called after the kernel configuration is done by
/// the bootloader specific code.
///
/// This function is only called once.
/// When this function is called, the `kernel_info` is setup with the correct information.
/// See [`RuntimeInfo`](crate::base::info::RuntimeInfo) for more information.
/// The heap is also setup, but the size can be non standard.
#[unsafe(no_mangle)]
extern "C" fn kernel_main(params: KernelParams) -> ! {
    let kernel_info = kernel_info();

    panic!("Params: {:#?}\nReached end of kernel", params);
}

#[cfg(test)]
mod tests {
    use core::panic::PanicInfo;

    #[unsafe(no_mangle)]
    extern "C" fn kernel_entry() -> ! {
        crate::kernel_entry()
    }

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
