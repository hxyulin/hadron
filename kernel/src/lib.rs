//! Hadron Kernel
#![no_std]
#![no_main]
#![feature(custom_test_frameworks, abi_x86_interrupt)]
#![test_runner(crate::tests::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![allow(unexpected_cfgs)]

use linked_list_allocator::LockedHeap;

pub mod base;
/// Boot shouldn't be accessible from the main kernel logic
pub(crate) mod boot;
pub mod devices;
pub mod util;

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
/// The logger should be set up, and the TTY devices should be added to the logger.
#[unsafe(no_mangle)]
extern "C" fn kernel_main(params: KernelParams) -> ! {
    log::info!("Hello from kernel!");

    panic!("Reached end of kernel");
}

#[cfg_attr(test, panic_handler)]
pub fn kernel_panic(info: &core::panic::PanicInfo) -> ! {
    if boot::is_boot() {
        boot::boot_panic(info);
    } else {
        log::error!("KERNEL PANIC: {}", info);
        loop {}
    }
}

#[cfg(test)]
mod tests {
    #[unsafe(no_mangle)]
    extern "C" fn kernel_entry() -> ! {
        crate::kernel_entry()
    }

    pub fn test_runner(tests: &[&dyn Fn()]) {
        for test in tests {
            test();
        }
    }
}
