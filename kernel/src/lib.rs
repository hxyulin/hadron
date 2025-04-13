//! Hadron Kernel
//! This contains the code of the core of the kernel
//! Things like builtin drivers, and modules are not included here
//! They are either loaded during runtime, or compiled into the kernel (still loaded at early boot)

#![no_std]
#![no_main]
#![feature(custom_test_frameworks, abi_x86_interrupt, allocator_api, vec_push_within_capacity, unsafe_cell_access)]
#![test_runner(crate::tests::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![allow(unexpected_cfgs, dead_code, clippy::new_without_default)]
// These features are needed for Arc
// We can remove them once they become stable
#![feature(unsize, dispatch_from_dyn, coerce_unsized)]

use base::{arch::x86_64::acpi, mem::allocator::KernelAllocator};

pub mod base;
pub mod boot;
pub mod util;

pub use boot::limine::limine_entry as kernel_entry;
use x86_64::PhysAddr;

extern crate alloc;

#[global_allocator]
pub static ALLOCATOR: KernelAllocator = KernelAllocator::empty();

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
extern "Rust" fn kernel_main(params: KernelParams) -> ! {
    log::debug!("initializing kernel");

    // Initialize ACPI info
    // This give us access to:
    // - HPET (or PS Timer)
    // - APICs (Local, IO)
    // - PCI devices
    acpi::init(params.rsdp);

    // Initialize the drivers for the kernel
    init_drivers();

    #[cfg(test)]
    hadron_test::exit_qemu(hadron_test::ExitCode::Success);
    panic!("reached end of kernel");
}

/// Initialize the drivers for the kernel
///
/// This involes:
/// - Finding the drivers for the devices
fn init_drivers() {
    log::debug!("initializing drivers");
}

#[cfg_attr(test, panic_handler)]
pub fn kernel_panic(info: &core::panic::PanicInfo) -> ! {
    x86_64::instructions::interrupts::disable();
    if boot::is_boot() {
        boot::boot_panic(info);
    } else {
        use crate::util::backtrace::panic_backtrace;
        panic_backtrace(info);
        loop {
            x86_64::instructions::hlt();
        }
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
