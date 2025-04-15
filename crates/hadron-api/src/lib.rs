//! Hadron API
//!
//! This crate provides the API for the Hadron kernel.
//! Which can be used to make drivers and other components.

#![no_std]

#[repr(C)]
pub struct SemVer {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

#[repr(C)]
pub struct DrmDriver {
    pub features: u32,
    /// The name of the driver
    pub name: &'static str,
    /// The description of the driver
    pub desc: &'static str,
    /// The version of the driver
    pub ver: SemVer,
}
