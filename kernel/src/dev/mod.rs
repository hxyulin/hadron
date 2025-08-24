//! Hadron Devices

use core::{ffi::c_void, ptr::NonNull};

use spin::{Mutex, MutexGuard};

use crate::dev::drivers::DriverCapabilities;

pub mod console;
pub mod drivers;
pub mod platform;

pub struct DeviceTree {
    platform: Mutex<platform::PlatformDeviceTree>,
}

impl DeviceTree {
    pub const fn empty() -> Self {
        Self {
            platform: Mutex::new(platform::PlatformDeviceTree::empty()),
        }
    }

    pub fn platform(&self) -> MutexGuard<'_, platform::PlatformDeviceTree> {
        self.platform.lock()
    }
}

#[derive(Debug)]
pub struct Device {
    pub drv: Option<DeviceDriver>,
}

#[derive(Debug)]
pub struct DeviceDriver {
    // TODO: Technically not safe, we should use some sort of custom structure that incapsulates
    // NonNull<c_void>
    pub data: Mutex<NonNull<c_void>>,
    pub caps: &'static DriverCapabilities,
}

unsafe impl Send for DeviceDriver {}
unsafe impl Sync for DeviceDriver {}

impl Device {
    pub fn new() -> Self {
        Self { drv: None }
    }
}

pub static DEVICES: DeviceTree = DeviceTree::empty();
