use core::{any::Any, fmt};

use alloc::{boxed::Box, sync::Arc, vec::Vec};

use crate::dev::Device;

#[derive(Debug)]
pub struct PlatformDeviceTree {
    devs: Vec<PlatformDev>,
}

impl PlatformDeviceTree {
    pub const fn empty() -> Self {
        Self { devs: Vec::new() }
    }

    pub fn add_device(&mut self, dev: PlatformDev) {
        self.devs.push(dev);
    }

    pub fn iter(&mut self) -> core::slice::Iter<'_, PlatformDev> {
        self.devs.iter()
    }

    pub fn iter_mut(&mut self) -> core::slice::IterMut<'_, PlatformDev> {
        self.devs.iter_mut()
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformDevType {
    IoDevice,
    Framebuffer,
}

#[repr(C)]
pub struct PlatformDev {
    pub name: &'static str,
    pub class: PlatformDevType,
    pub dev: Arc<Device>,
    pub addr: PlatformDevAddr,
}

impl fmt::Debug for PlatformDev {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PlatformDev")
            .field("name", &self.name)
            .field("class", &self.class)
            .field("dev", &self.dev)
            .finish()
    }
}

impl PlatformDev {
    pub fn new(name: &'static str, class: PlatformDevType, addr: PlatformDevAddr) -> Self {
        Self {
            name,
            class,
            dev: Arc::new(Device::new()),
            addr,
        }
    }
}

#[repr(C)]
pub struct PlatformDevMatcher {
    pub name: &'static str,
    pub addr: Option<PlatformDevAddr>,
}

impl PlatformDevMatcher {
    pub fn matches(&self, dev: &PlatformDev) -> bool {
        // FIXME: For now, we only check IO_Port
        self.name == dev.name
            && self
                .addr
                .as_ref()
                .map(|addr| unsafe { addr.io_port == dev.addr.io_port })
                .unwrap_or(true)
    }
}

impl core::fmt::Debug for PlatformDevMatcher {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PlatformDevMatcher").field("name", &self.name).finish()
    }
}

#[repr(C)]
pub union PlatformDevAddr {
    pub io_port: u16,
    pub addr: usize,
    pub bytes: [u8; 8],
}

impl PlatformDevAddr {
    pub const fn io_port(io_port: u16) -> Self {
        Self { io_port }
    }

    pub const fn addr(addr: usize) -> Self {
        Self { addr }
    }
}
