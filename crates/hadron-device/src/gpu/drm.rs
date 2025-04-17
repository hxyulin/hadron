//! The Direct Render Manager (DRM)
//!
use hadron_base::util::version::SemVer;

#[repr(C)]
#[derive(Debug)]
pub struct DrmDriver {
    pub features: DrmFeatures,
    /// The name of the driver
    pub name: &'static str,
    /// The description of the driver
    pub desc: &'static str,
    /// The version of the driver
    pub ver: SemVer,
}

bitflags::bitflags! {
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy)]
    pub struct DrmFeatures: u32 {
    }
}

/// The base DRM device
pub struct DrmDevice {}

pub struct DrmModeConfig {}

pub struct DrmPlane {}

pub struct DrmCrtc {}

pub struct DrmEncoder {}

pub struct DrmConnector {}
