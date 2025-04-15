use alloc::vec::Vec;

mod pcie;
pub use pcie::*;

use super::{Device, DeviceClass};

/// A PCI device
///
/// This is a device that is connected to the PCI bus or PCI-E bus
/// Despite the name, this is actually a PCI function
#[derive(Debug)]
pub struct PCIDev {
    /// The Revision ID of the device
    pub revision: u8,
    /// The BAR (Base Address Register) of the device
    ///
    /// This is an array of values that store the physical address of
    /// memory regions or MMIO regions of the device
    pub bars: [u32; 6],
    pub dev: Device,
}

#[derive(Clone, Copy, Eq, PartialOrd, Ord)]
pub struct PCIDeviceId {
    /// The vendor ID of the device, use 0xFFFF for any vendor
    pub vendor: u16,
    pub device: u16,
    pub subvendor: u16,
    pub subdevice: u16,
    pub class: DeviceClass,
    pub subclass: u8,
    /// User provided data
    pub data: &'static [u8],
}

impl PartialEq for PCIDeviceId {
    fn eq(&self, other: &Self) -> bool {
        let vendor_matches = self.vendor == other.vendor
            || self.vendor == PCIDeviceId::ANY_VENDOR
            || other.vendor == PCIDeviceId::ANY_VENDOR;
        let device_matches = self.device == other.device
            || self.device == PCIDeviceId::ANY_DEVICE
            || other.device == PCIDeviceId::ANY_DEVICE;
        let subvendor_matches = self.subvendor == other.subvendor
            || self.subvendor == PCIDeviceId::ANY_SUBVENDOR
            || other.subvendor == PCIDeviceId::ANY_SUBVENDOR;
        let subdevice_matches = self.subdevice == other.subdevice
            || self.subdevice == PCIDeviceId::ANY_SUBDEVICE
            || other.subdevice == PCIDeviceId::ANY_SUBDEVICE;
        let class_matches =
            self.class == other.class || self.class == PCIDeviceId::ANY_CLASS || other.class == PCIDeviceId::ANY_CLASS;
        let subclass_matches = self.subclass == other.subclass
            || self.subclass == PCIDeviceId::ANY_SUBCLASS
            || other.subclass == PCIDeviceId::ANY_SUBCLASS;
        // We don't care about the data
        vendor_matches && device_matches && subvendor_matches && subdevice_matches && class_matches && subclass_matches
    }
}

impl Default for PCIDeviceId {
    fn default() -> Self {
        Self {
            vendor: PCIDeviceId::ANY_VENDOR,
            device: PCIDeviceId::ANY_DEVICE,
            subvendor: PCIDeviceId::ANY_SUBVENDOR,
            subdevice: PCIDeviceId::ANY_SUBDEVICE,
            class: PCIDeviceId::ANY_CLASS,
            subclass: 0,
            data: &[],
        }
    }
}

impl PCIDeviceId {
    pub const ANY_VENDOR: u16 = 0xFFFF;
    pub const ANY_SUBVENDOR: u16 = 0xFFFF;
    pub const ANY_DEVICE: u16 = 0xFFFF;
    pub const ANY_SUBDEVICE: u16 = 0xFFFF;
    pub const ANY_CLASS: DeviceClass = DeviceClass::Unknown(0xFF);
    pub const ANY_SUBCLASS: u8 = 0xFF;
}

impl core::fmt::Debug for PCIDeviceId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PCIDeviceId")
            .field("vendor", &format_args!("{:#x}", self.vendor))
            .field("device", &format_args!("{:#x}", self.device))
            .field("subvendor", &format_args!("{:#x}", self.subvendor))
            .field("subdevice", &format_args!("{:#x}", self.subdevice))
            .field("class", &self.class)
            .field("subclass", &self.subclass)
            .finish()
    }
}

#[derive(Debug)]
pub struct PCIDeviceTree {
    root: PCIDeviceTreeNode,
}

impl PCIDeviceTree {
    pub fn iter(&self) -> PCIDeviceTreeIter {
        PCIDeviceTreeIter::new(&self.root)
    }
}

pub struct PCIDeviceTreeIter<'a> {
    stack: Vec<&'a PCIDeviceTreeNode>,
    function_index: usize,
}

impl<'a> PCIDeviceTreeIter<'a> {
    pub fn new(root: &'a PCIDeviceTreeNode) -> Self {
        Self {
            stack: alloc::vec![root],
            function_index: 0,
        }
    }
}

type PCIFunction = (PCIDev, PCIDeviceId);

impl<'a> Iterator for PCIDeviceTreeIter<'a> {
    type Item = &'a PCIFunction;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(node) = self.stack.pop() {
            match &node {
                PCIDeviceTreeNode::Bus(bus) => {
                    for device in &bus.devices {
                        self.stack.push(device);
                    }
                }
                PCIDeviceTreeNode::Device(device) => {
                    if self.function_index < device.functions.len() {
                        self.stack.push(&node);
                        self.function_index += 1;
                        return Some(&device.functions[self.function_index - 1]);
                    } else {
                        self.function_index = 0;
                    }
                }
            }
        }
        None
    }
}

pub enum PCIDeviceTreeNode {
    Bus(PCIBusDevice),
    Device(PCIDevice),
}

impl core::fmt::Debug for PCIDeviceTreeNode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            PCIDeviceTreeNode::Bus(bus) => bus.fmt(f),
            PCIDeviceTreeNode::Device(device) => device.fmt(f),
        }
    }
}

#[derive(Debug)]
pub struct PCIBusDevice {
    bus_number: u8,
    devices: Vec<PCIDeviceTreeNode>,
}

#[derive(Debug)]
pub struct PCIDevice {
    device_number: u8,
    functions: Vec<PCIFunction>,
}

impl PCIDevice {
    pub const fn empty(device: u8) -> Self {
        Self {
            device_number: device,
            functions: Vec::new(),
        }
    }
}
