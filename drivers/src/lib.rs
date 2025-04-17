//! The builtin drivers for the Hadron kernel

#![no_std]

use hadron_base::{
    dev::gpu::drm::{DrmDriver, DrmFeatures},
    util::version::SemVer,
};

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
