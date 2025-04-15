//! Devices
//!
//! The device / driver model is inspired by the Linux kernel
//! The device registry contains all devices that are connected to the system, each bus has its own
//! tree of devices.
//!
//! The drivers are loaded at boot time, and are responsible for initializing the devices. Drivers
//! can be initialized from PCI devices, and will create a [`Devoce`](crate::dev::drivers::Device)
//! for each device.

use core::{alloc::Allocator, ptr::NonNull};

use crate::base::mem::sync::UninitMutex;

pub mod gpu;
pub mod helpers;
pub mod pci;

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
pub static DEVICES: UninitMutex<DeviceRegistry> = UninitMutex::uninit();

/// A centralized registry of devices
///
/// This stores all devices that are connected to the system
pub struct DeviceRegistry {
    /// Represents devices on the PCI bus
    pub pci: pci::PCIDeviceTree,
}

/// The base device
#[derive(Debug)]
pub struct Device {
    /// A managed allocator for the device
    ///
    /// All allocations made by the device will be allocated from this allocator
    /// This ensures that memory allocated will be deallocated when the device is dropped
    allocator: DeviceAllocator,

    /// Driver data for the device
    ///
    /// Drivers should only store their data here, and avoid using any state anywhere else
    driver_data: Option<NonNull<core::ffi::c_void>>,
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
