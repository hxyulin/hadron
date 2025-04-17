//! The builtin drivers for the Hadron kernel

#![no_std]

use hadron_base::util::version::SemVer;
use hadron_device::gpu::drm::{DrmDriver, DrmFeatures};

/// A special symbol that is used so that the linker actually looks into the .rlib
/// for the object files, to resolve this symbol. Otherwise this crate won't be linked
/// (because it is not designed to be used and exposed to rust as a crate).
#[doc(hidden)]
#[used]
#[unsafe(export_name = "INCLUDE_DRV_DRIVERS")]
pub static __HIDDEN: u8 = 0;

#[used]
#[unsafe(export_name = "TEST")]
#[unsafe(link_section = ".drm_drivers")]
static BOCHS_VGA: DrmDriver = DrmDriver {
    features: DrmFeatures::empty(),
    name: "Bochs VGA",
    desc: "The VGA driver for Bochs / QEMU",
    ver: SemVer::new(0, 0, 1),
};

pub fn pci_drivers() -> &'static [DrmDriver] {
    unsafe extern "C" {
        static _drm_drv_start: u8;
        static _drm_drv_end: u8;
    }
    let start = &raw const _drm_drv_start as usize;
    let end = &raw const _drm_drv_end as usize;
    let count = (end - start) / core::mem::size_of::<DrmDriver>();
    let start = &raw const _drm_drv_start as *const DrmDriver;
    unsafe { core::slice::from_raw_parts(start, count) }
}
