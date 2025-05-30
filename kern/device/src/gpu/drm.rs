//! The Direct Render Manager (DRM)

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
