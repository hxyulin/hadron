#![no_std]
#![feature(allocator_api, vec_push_within_capacity)]

use hadron_base::{
    KernelParams,
    base::{arch::x86_64::acpi, mem::allocator::KernelAllocator},
    dev::gpu::drm::DrmDriver,
};

pub mod boot;

#[global_allocator]
pub static ALLOCATOR: KernelAllocator = KernelAllocator::empty();

extern crate alloc;

// We need to use extern crate to allow the linker to find the symbols
// If we remove this, the .drivers section will be empty
extern crate hadron_drivers;

pub use boot::limine::limine_entry as kernel_entry;

/// The main kernel 'entry point'
/// It is sort of an intermediate stage, which is called after the kernel configuration is done by
/// the bootloader specific code.
///
/// This function is only called once.
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
    unsafe extern "C" {
        static _drm_drv_start: u8;
        static _drm_drv_end: u8;
    }
    let start = &raw const _drm_drv_start as usize;
    let end = &raw const _drm_drv_end as usize;
    let count = (end - start) / core::mem::size_of::<DrmDriver>();
    let start = &raw const _drm_drv_start as *const DrmDriver;
    for i in 0..count {
        let drv = unsafe { &*(start.add(i)) };
        log::debug!("drv: {:#?}", drv);
    }
    log::debug!("CPU Features: {:#?}", hadron_base::util::cpu::cpu_features());
}

#[cfg_attr(test, panic_handler)]
pub fn kernel_panic(info: &core::panic::PanicInfo) -> ! {
    x86_64::instructions::interrupts::disable();
    if boot::is_boot() {
        boot::boot_panic(info);
    } else {
        use hadron_base::util::backtrace::panic_backtrace;
        panic_backtrace(info);
        loop {
            x86_64::instructions::hlt();
        }
    }
}
