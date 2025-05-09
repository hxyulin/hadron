#![no_std]
#![no_main]
#![feature(custom_test_frameworks, allocator_api, vec_push_within_capacity)]
#![test_runner(crate::tests::test_runner)]
#![reexport_test_harness_main = "test_main"]

use alloc::vec::Vec;
use hadron_base::{
    KernelParams,
    base::arch::x86_64::acpi::{self, PCIeBusRegion},
};
use hadron_device::pci::PCIeConfigSpace;

pub mod boot;

extern crate alloc;

// We need to use extern crate to allow the linker to find the symbols
// If we remove this, the .drivers section will be empty if we don't use any exported symbols
// To be safe, we just mark it as extern crate
extern crate hadron_drivers;

pub use boot::limine::limine_entry as kernel_entry;

/// The main kernel 'entry point'
/// It is sort of an intermediate stage, which is called after the kernel configuration is done by
/// the bootloader specific code.
///
/// This function is only called once.
#[unsafe(no_mangle)]
extern "Rust" fn kernel_main(params: KernelParams) -> ! {
    let span = tracing::span!(tracing::Level::TRACE, "kernel_main");
    let _enter = span.enter();

    log::debug!("initializing kernel");

    // Initialize ACPI info
    // This give us access to:
    // - HPET (or PS Timer)
    // - APICs (Local, IO)
    // - PCI devices
    let acpi = acpi::init(params.rsdp);
    init_devices(acpi.pcie_regions);

    // Initialize the drivers for the kernel
    init_drivers();

    #[cfg(test)]
    hadron_test::exit_qemu(hadron_test::ExitCode::Success);
    panic!("reached end of kernel");
}

/// Initialize the devices for the kernel
fn init_devices(pcie_regions: Vec<PCIeBusRegion>) {
    use hadron_device::DEVICES;
    let spaces = pcie_regions
        .into_iter()
        .map(|r| PCIeConfigSpace::identity_mapped(r.base_address, r.bus_range))
        .collect();
    let pci = hadron_device::pci::PCIDeviceTree::from_pcie(spaces);
    DEVICES.write().pci = pci;
}

/// Initialize the drivers for the kernel
///
/// This involes:
/// - Finding the drivers for the devices
fn init_drivers() {
    for pci_driver in hadron_drivers::pci_drivers() {
        log::debug!("Driver: {:?}", pci_driver);
    }
    for platform_driver in hadron_drivers::platform_drivers() {
        log::debug!("Driver: {:?}", platform_driver);
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
        log::logger().flush();
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
