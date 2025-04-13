use spin::Mutex;

use crate::base::{pci::PCIeDeviceInfo, mem::Arc};

pub mod qemu;

/// A unique identifier for a device
/// This consists of a vendor ID and a device ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct DeviceId(u32);

impl DeviceId {
    pub const fn from_parts(vendor_id: u16, device_id: u16) -> Self {
        Self((vendor_id as u32) << 16 | (device_id as u32))
    }

    pub fn vendor(&self) -> u16 {
        (self.0 >> 16) as u16
    }

    pub fn device(&self) -> u16 {
        (self.0 & 0xFFFF) as u16
    }
}

impl core::fmt::Display for DeviceId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:#x}:{:#x}", self.vendor(), self.device())
    }
}

pub struct DriverMeta {
    pub name: &'static str,
    pub version: &'static str,
    pub supported_devices: &'static [DeviceId],
    pub create: fn(PCIeDeviceInfo) -> Arc<Mutex<dyn Driver>>,
}

pub trait Driver {}

static DRIVERS: &[DriverMeta] = &[DriverMeta {
    name: "QEMU VGA",
    version: "0.1.0",
    supported_devices: &[DeviceId::from_parts(0x1234, 0x1111)],
    create: qemu::vga::Vga::create,
}];

pub fn find_driver(device_id: DeviceId) -> Option<&'static DriverMeta> {
    DRIVERS.iter().find(|d| d.supported_devices.contains(&device_id))
}
