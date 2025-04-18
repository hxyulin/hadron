//! Devices
//!
//! The device / driver model is inspired by the Linux kernel
//! The device registry contains all devices that are connected to the system, each bus has its own
//! tree of devices.
//!
//! The drivers are loaded at boot time, and are responsible for initializing the devices. Drivers
//! can be initialized from PCI devices, and will create a [`Devoce`](crate::dev::drivers::Device)
//! for each device.

#![no_std]
#![feature(allocator_api)]

extern crate alloc;

use core::{alloc::Allocator, ptr::NonNull};

use alloc::vec::Vec;
use hadron_base::base::mem::sync::UninitMutex;
use spin::RwLock;

pub mod gpu;
pub mod helpers;
pub mod mem;

pub mod pci;
pub mod platform;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DeviceClass {
    /// Unclassified device
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
    /// Unassigned device
    Unassigned,
    Unknown(u8),
}

/// The device registry
///
/// See [`DeviceRegistry`](crate::dev::DeviceRegistry) for more information
pub static DEVICES: RwLock<DeviceRegistry> = RwLock::new(DeviceRegistry::empty());

/// A centralized registry of devices
///
/// This stores all devices that are connected to the system
pub struct DeviceRegistry {
    /// Represents devices on the PCI bus
    pub pci: pci::PCIDeviceTree,
    pub platform: platform::PlatformDeviceTree,
}

// For now platform devices are just a list

impl DeviceRegistry {
    pub const fn empty() -> Self {
        Self {
            pci: pci::PCIDeviceTree::empty(),
            platform: platform::PlatformDeviceTree::empty(),
        }
    }
}

/// The base device
#[derive(Debug)]
pub struct Device {
    /// A managed allocator for the device
    ///
    /// All allocations made by the device will be allocated from this allocator
    /// This ensures that memory allocated will be deallocated when the device is dropped
    pub allocator: DeviceAllocator,

    /// A memory mapper for the device
    ///
    /// All memory mapped regions should be done using this mapper
    pub mapper: DeviceMapper,

    /// Driver data for the device
    ///
    /// Drivers should only store their data here, and avoid using any state anywhere else
    pub driver_data: Option<NonNull<core::ffi::c_void>>,

    /// The device's virtual table
    ///
    /// This contains generic device functions
    /// TODO: Maybe we should combine this with the driver data, so that we have a unified data structure
    /// that is optional
    pub vtable: Option<DeviceVTable>,
}

impl Device {
    pub fn new() -> Self {
        Self {
            allocator: DeviceAllocator {},
            mapper: DeviceMapper {
                mapped_regions: Vec::new(),
            },
            driver_data: None,
            vtable: None,
        }
    }
}

#[derive(Debug)]
pub struct DeviceVTable {}

// TODO: Make it actually safe to share across cores
unsafe impl Send for Device {}
unsafe impl Sync for Device {}

#[derive(Debug)]
struct DeviceMapper {
    mapped_regions: Vec<mem::MMRegion>,
}

#[derive(Debug)]
struct DeviceAllocator {}

unsafe impl Allocator for DeviceAllocator {
    fn allocate(&self, layout: core::alloc::Layout) -> Result<core::ptr::NonNull<[u8]>, core::alloc::AllocError> {
        alloc::alloc::Global.allocate(layout)
    }

    unsafe fn deallocate(&self, ptr: core::ptr::NonNull<u8>, layout: core::alloc::Layout) {
        unsafe { alloc::alloc::Global.deallocate(ptr, layout) }
    }
}
