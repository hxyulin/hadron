//! The Direct Render Manager (DRM)
//!
use crate::util::version::SemVer;

pub struct DrmDriver {
    features: DrmFeatures,
    /// The name of the driver
    name: &'static str,
    /// The description of the driver
    desc: &'static str,
    /// The version of the driver
    ver: SemVer,
}

bitflags::bitflags! {
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
