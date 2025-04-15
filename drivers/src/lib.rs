//! The builtin drivers for the Hadron kernel

#![no_std]

use hadron_base::{
    dev::gpu::drm::{DrmDriver, DrmFeatures},
    util::version::SemVer,
};

#[doc(hidden)]
#[used]
#[unsafe(export_name = "INCLUDE_DRM_DRIVERS")]
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
