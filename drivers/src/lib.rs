//! The builtin drivers for the Hadron kernel

#![no_std]

use hadron_api::DrmDriver;

#[used]
#[unsafe(export_name = "INCLUDE_DRM_DRIVERS")]
pub static __HIDDEN: u8 = 0;

#[used]
#[unsafe(export_name = "TEST")]
#[unsafe(link_section = ".drm_drivers")]
static BOCHS_VGA: DrmDriver = DrmDriver {
    features: 0,
    name: "Bochs VGA",
    desc: "The VGA driver for Bochs / QEMU",
    ver: hadron_api::SemVer {
        major: 0,
        minor: 0,
        patch: 1,
    },
};
