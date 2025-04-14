//! Devices

use alloc::vec::Vec;
use drivers::LoadedDriver;

use crate::base::mem::sync::UninitMutex;

pub mod drivers;
pub mod helpers;
pub mod pci;

pub static DEVICES: UninitMutex<DeviceTree> = UninitMutex::uninit();

#[derive(Debug)]
pub struct DeviceTree {
    root: DeviceTreeNode,
}

impl DeviceTree {
    pub fn iter(&self) -> DeviceTreeIter {
        DeviceTreeIter::new(&self.root)
    }
}

pub struct DeviceTreeIter<'a> {
    stack: Vec<&'a DeviceTreeNode>,
}

impl<'a> DeviceTreeIter<'a> {
    pub fn new(root: &'a DeviceTreeNode) -> Self {
        use alloc::vec;
        Self { stack: vec![root] }
    }
}

impl<'a> Iterator for DeviceTreeIter<'a> {
    type Item = &'a DeviceFunction;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(node) = self.stack.pop() {
            match node {
                DeviceTreeNode::Bus(bus) => {
                    for device in &bus.devices {
                        self.stack.push(device);
                    }
                }
                DeviceTreeNode::Device(device) => {
                    return Some(&device.functions[0]);
                }
            }
        }
        None
    }
}

pub enum DeviceTreeNode {
    Bus(BusDevice),
    Device(Device),
}

#[derive(Debug)]
pub struct BusDevice {
    bus: u8,
    devices: Vec<DeviceTreeNode>,
}

#[derive(Debug)]
pub struct Device {
    device: u8,
    functions: Vec<DeviceFunction>,
}

impl Device {
    pub const fn empty(device: u8) -> Self {
        Self {
            device,
            functions: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct DeviceId(u32);

impl DeviceId {
    pub const fn new(vendor_id: u16, device_id: u16) -> Self {
        Self((vendor_id as u32) << 16 | device_id as u32)
    }

    pub const fn vendor_id(&self) -> u16 {
        (self.0 >> 16) as u16
    }

    pub const fn device_id(&self) -> u16 {
        self.0 as u16
    }
}

impl core::fmt::Debug for DeviceId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:#x}:{:#x}", self.vendor_id(), self.device_id())
    }
}

pub struct DeviceFunction {
    pub inner: DeviceFunctionInner,
    pub driver: Option<&'static LoadedDriver>,
}

pub struct DeviceFunctionInner {
    function: u8,
    vendor_id: u16,
    device_id: u16,
    revision: u8,
    class: DeviceClass,
    bars: [u32; 6],
}

impl DeviceFunctionInner {
    pub fn id(&self) -> DeviceId {
        DeviceId::new(self.vendor_id, self.device_id)
    }

    pub fn class(&self) -> DeviceClass {
        self.class
    }
}

impl DeviceFunction {
    pub fn id(&self) -> DeviceId {
        self.inner.id()
    }
}

impl core::fmt::Debug for DeviceTreeNode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            DeviceTreeNode::Bus(bus) => bus.fmt(f),
            DeviceTreeNode::Device(device) => device.fmt(f),
        }
    }
}

impl core::fmt::Debug for DeviceFunctionInner {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("DeviceFunction")
            .field("function", &self.function)
            .field("vendor_id", &format_args!("{:#x}", self.vendor_id))
            .field("device_id", &format_args!("{:#x}", self.device_id))
            .field("revision", &self.revision)
            .field("class", &self.class)
            .field("bars", &self.bars)
            .finish()
    }
}

impl core::fmt::Debug for DeviceFunction {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("DeviceFunction")
            .field("inner", &self.inner)
            .finish()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DeviceClass {
    Unclassified,
    MassStorageController,
    NetworkController,
    DisplayController,
    MultimediaController,
    MemoryController,
    BridgeDevice,
    SimpleCommunicationController,
    BaseSystemPeripheral,
    InputDeviceController,
    DockingStation,
    Processor,
    SerialBusController,
    WirelessController,
    IntelligentController,
    SatelliteCommunicationController,
    EncryptionController,
    SignalProcessingController,
    ProcessingAccelerator,
    NonEssentialInstrumentation,
    Coprocessor,
    Unassigned,
    Unknown(u8),
}

