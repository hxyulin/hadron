//! The builtin drivers for the Hadron kernel
//!
//! Different types of drivers are built into different sections of the kernel.
//! Here is a list of the sections:
//! - `pci_drivers`: PCI drivers
//! - `platform_drivers`: Platform drivers

#![no_std]

use hadron_driver_api::{SemVer, pci::PCIDriver, platform::PlatformDriver};

mod gpu;

/// A special symbol that is used so that the linker actually looks into the .rlib
/// for the object files, to resolve this symbol. Otherwise this crate won't be linked
/// (because it is not designed to be used and exposed to rust as a crate).
#[doc(hidden)]
#[used]
#[unsafe(export_name = "INCLUDE_DEV_DRIVERS")]
pub static __HIDDEN: u8 = 0;

pub fn pci_drivers() -> &'static [PCIDriver] {
    unsafe extern "C" {
        static _pci_drv_start: u8;
        static _pci_drv_end: u8;
    }
    let start = &raw const _pci_drv_start as usize;
    let end = &raw const _pci_drv_end as usize;
    let count = (end - start) / core::mem::size_of::<PCIDriver>();
    let start = &raw const _pci_drv_start as *const PCIDriver;
    unsafe { core::slice::from_raw_parts(start, count) }
}

pub fn platform_drivers() -> &'static [PlatformDriver] {
    unsafe extern "C" {
        static _platform_drv_start: u8;
        static _platform_drv_end: u8;
    }
    let start = &raw const _platform_drv_start as usize;
    let end = &raw const _platform_drv_end as usize;
    let count = (end - start) / core::mem::size_of::<PlatformDriver>();
    let start = &raw const _platform_drv_start as *const PlatformDriver;
    unsafe { core::slice::from_raw_parts(start, count) }
}
