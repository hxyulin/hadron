use alloc::vec::Vec;
use hadron_base::util::typing::AnyVec;

use crate::Device;

pub enum PlatformDevClass {
    Framebuffer,
}

pub struct PlatformDeviceTree {
    devices: Vec<PlatformDev>,
}

impl PlatformDeviceTree {
    pub const fn empty() -> Self {
        Self { devices: Vec::new() }
    }

    fn add_device(&mut self, dev: PlatformDev) {
        self.devices.push(dev);
    }
}

pub struct PlatformDev {
    /// The compatible string of the device
    pub name: &'static str,
    pub class: PlatformDevClass,
    pub dev: Device,
    /// Data for the device, specific to the driver
    pub data: AnyVec,
}

#[derive(Debug)]
pub struct PlatformDriver {
    pub name: &'static str,
    pub matches: &'static [PlatformDrvMatch],
    pub probe: fn(&PlatformDev) -> u32,
}

#[derive(Debug)]
pub struct PlatformDrvMatch {
    /// The name of the driver (for matching)
    pub compatible: &'static str,
}
