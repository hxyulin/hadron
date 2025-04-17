use core::ops::{DerefMut, RangeInclusive};

use alloc::vec::Vec;
use x86_64::{
    PhysAddr, VirtAddr,
    structures::paging::{Page, PageSize, PageTableFlags, PhysFrame, Size2MiB},
};

use hadron_base::base::mem::page_table::PageTable;
use crate::DeviceClass;

/// The size of a PCIe device in bytes
const DEVICE_SIZE: u64 = 8 * 4096;
/// The size of a PCIe bus in bytes
const BUS_SIZE: u64 = 32 * DEVICE_SIZE;

/// A PCIe config space
///
/// This is a region of contiguous physical memory that is mapped to a virtual address.
/// The virtual address is aligned to 2MiB.
/// There can be multiple config spaces, but the bus numbers should not overlap.
pub struct PCIeConfigSpace {
    /// The physical base address of the config space
    phys_base: PhysAddr,
    /// THe mapped virtual address of the config space
    virt_base: VirtAddr,
    /// The bus numbers that this config space is mapped to
    buses: RangeInclusive<u8>,
}

impl PCIeConfigSpace {
    pub fn identity_mapped(base: PhysAddr, buses: RangeInclusive<u8>) -> Self {
        let size = (*buses.end() as u64 - *buses.start() as u64 + 1) * BUS_SIZE;
        let virt_base = Self::map_with_offset(base, 0, size);
        Self {
            phys_base: base,
            virt_base,
            buses,
        }
    }

    fn map_with_offset(base: PhysAddr, offset: u64, size: u64) -> VirtAddr {
        assert!(
            base.as_u64() % Size2MiB::SIZE == 0,
            "base address must be aligned to 2MiB"
        );
        assert!(offset % Size2MiB::SIZE == 0, "offset must be aligned to 2MiB");
        let pages = size.div_ceil(Size2MiB::SIZE);
        log::debug!("PCI: mapping config space ({} 2MiB pages)", pages);
        let mut page_table = hadron_base::base::mem::PAGE_TABLE.lock();
        let mut allocator = hadron_base::base::mem::FRAME_ALLOCATOR.lock();
        let flags =
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_EXECUTE | PageTableFlags::NO_CACHE;
        for i in 0..pages {
            let addr = base + i * Size2MiB::SIZE;
            unsafe {
                page_table.map_with_allocator(
                    Page::<Size2MiB>::from_start_address_unchecked(VirtAddr::new(addr.as_u64())),
                    PhysFrame::<Size2MiB>::from_start_address_unchecked(addr),
                    flags,
                    allocator.deref_mut(),
                )
            };
        }
        VirtAddr::new(base.as_u64() + offset)
    }

    pub fn size(&self) -> u64 {
        (*self.buses.end() as u64 - *self.buses.start() as u64 + 1) * BUS_SIZE
    }

    pub fn contains_bus(&self, bus: u8) -> bool {
        self.buses.contains(&bus)
    }
}

impl Drop for PCIeConfigSpace {
    fn drop(&mut self) {
        let mut page_table = hadron_base::base::mem::PAGE_TABLE.lock();
        let pages = self.size().div_ceil(Size2MiB::SIZE);
        log::debug!("PCI: unmapping config space ({} 2MiB pages)", pages);
        for i in 0..pages {
            let addr = self.phys_base + i * Size2MiB::SIZE;
            unsafe {
                page_table.unmap(Page::<Size2MiB>::from_start_address_unchecked(VirtAddr::new(
                    addr.as_u64(),
                )))
            };
        }
    }
}

pub struct PCIeBus {
    bus: u8,
}

impl PCIeBus {
    pub const fn new(bus: u8) -> Self {
        Self { bus }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PCIEFunctionAddressType {
    Memory,
    IO,
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct PCICapabilities: u32 {
        const PMCAP = 1 << 0;  // Power Management Capability
        const MSICAP = 1 << 1; // MSI Capability
        const MSIXCAP = 1 << 2; // MSI-X Capability
        const PCIECAP = 1 << 3; // PCIe Capability
    }
}

/// A PCIe function
///
/// This is named `PCIeDevice` because functions are usually referred to as devices
struct PCIeDevice {
    /// The base address of the PCI function
    base: VirtAddr,
}

impl PCIeDevice {
    pub const fn new(base: VirtAddr) -> Self {
        Self { base }
    }

    pub fn read<T>(&self, offset: PCIeFunctionOffset) -> T {
        unsafe { self.read_internal::<T>(offset as u16) }
    }

    pub fn write<T>(&mut self, offset: PCIeFunctionOffset, value: T) {
        unsafe { self.write_internal::<T>(offset as u16, value) };
    }

    unsafe fn read_internal<T>(&self, offset: u16) -> T {
        unsafe { (self.base + offset as u64).as_ptr::<T>().read_volatile() }
    }

    unsafe fn write_internal<T>(&mut self, offset: u16, value: T) {
        unsafe { (self.base + offset as u64).as_mut_ptr::<T>().write_volatile(value) }
    }

    pub fn class(&self) -> DeviceClass {
        let class = self.read::<u8>(PCIeFunctionOffset::Class);
        DeviceClass::from_u8(class)
    }

    pub fn subclass(&self) -> u8 {
        self.read::<u8>(PCIeFunctionOffset::Subclass)
    }

    pub fn subvendor(&self) -> u16 {
        self.read::<u16>(PCIeFunctionOffset::SubsystemVendorID)
    }

    pub fn subsystem(&self) -> u16 {
        self.read::<u16>(PCIeFunctionOffset::SubsystemID)
    }

    pub fn capabilities(&self) -> PCICapabilities {
        let mut capabilities = PCICapabilities::empty();

        let mut current_offset = self.read::<u16>(PCIeFunctionOffset::CapabilitiesPointer);

        while current_offset != 0 {
            // Read the Capability ID and the Next Capability Pointer
            let cap_id = unsafe { self.read_internal::<u32>(current_offset) & 0xFF };
            let next_pointer = unsafe { (self.read_internal::<u32>(current_offset) >> 8) & 0xFF };

            // Match the capability ID and add the corresponding flag
            // Attempt to convert to a PCIEFunctionCapabilities enum
            let cap = PCICapabilities::from_bits(cap_id);
            if let Some(cap) = cap {
                capabilities |= cap;
            }

            // Move to the next capability
            current_offset = next_pointer as u16;
        }

        capabilities
    }

    pub fn get_bars(&self) -> [u32; 6] {
        let mut bars = [0; 6];
        for i in 0..6 {
            // SAFETY: The offset is valid because it is within the range of the function
            let addr = unsafe { self.read_internal::<u32>(PCIeFunctionOffset::Bar0 as u16 + i * 4) };
            // When reading from a BAR, we need mask out the lower bits
            bars[i as usize] = addr & !0xF;
        }
        bars
    }
}

#[repr(u16)]
#[derive(Debug, Clone, Copy)]
pub enum PCIeFunctionOffset {
    VendorID = 0x00,
    DeviceID = 0x02,
    Command = 0x04,
    Status = 0x06,
    RevisionID = 0x08,
    ProgIF = 0x09,
    Subclass = 0x0A,
    Class = 0x0B,
    CacheLineSize = 0x0C,
    LatencyTimer = 0x0D,
    HeaderType = 0x0E,
    BIST = 0x0F,
    Bar0 = 0x10,
    Bar1 = 0x14,
    Bar2 = 0x18,
    Bar3 = 0x1C,
    Bar4 = 0x20,
    Bar5 = 0x24,
    CardbusCISPointer = 0x28,
    SubsystemVendorID = 0x2C,
    SubsystemID = 0x2E,
    ExpansionROMBaseAddress = 0x30,
    CapabilitiesPointer = 0x34,
    InterruptLine = 0x3C,
    InterruptPin = 0x3D,
    MinGrant = 0x3E,
    MaxLatency = 0x3F,
}

impl DeviceClass {
    pub fn from_u8(value: u8) -> DeviceClass {
        match value {
            0x00 => DeviceClass::Unclassified,
            0x01 => DeviceClass::MassStorageController,
            0x02 => DeviceClass::NetworkController,
            0x03 => DeviceClass::DisplayController,
            0x04 => DeviceClass::MultimediaController,
            0x05 => DeviceClass::MemoryController,
            0x06 => DeviceClass::BridgeDevice,
            0x07 => DeviceClass::SimpleCommunicationController,
            0x08 => DeviceClass::BaseSystemPeripheral,
            0x09 => DeviceClass::InputDeviceController,
            0x0A => DeviceClass::DockingStation,
            0x0B => DeviceClass::Processor,
            0x0C => DeviceClass::SerialBusController,
            0x0D => DeviceClass::WirelessController,
            0x0E => DeviceClass::IntelligentController,
            0x0F => DeviceClass::SatelliteCommunicationController,
            0x10 => DeviceClass::EncryptionController,
            0x11 => DeviceClass::SignalProcessingController,
            0x12 => DeviceClass::ProcessingAccelerator,
            0x13 => DeviceClass::NonEssentialInstrumentation,
            0x40 => DeviceClass::Coprocessor,
            0xFF => DeviceClass::Unassigned,
            other => DeviceClass::Unknown(other),
        }
    }

    pub fn to_u8(self) -> u8 {
        match self {
            DeviceClass::Unclassified => 0x00,
            DeviceClass::MassStorageController => 0x01,
            DeviceClass::NetworkController => 0x02,
            DeviceClass::DisplayController => 0x03,
            DeviceClass::MultimediaController => 0x04,
            DeviceClass::MemoryController => 0x05,
            DeviceClass::BridgeDevice => 0x06,
            DeviceClass::SimpleCommunicationController => 0x07,
            DeviceClass::BaseSystemPeripheral => 0x08,
            DeviceClass::InputDeviceController => 0x09,
            DeviceClass::DockingStation => 0x0A,
            DeviceClass::Processor => 0x0B,
            DeviceClass::SerialBusController => 0x0C,
            DeviceClass::WirelessController => 0x0D,
            DeviceClass::IntelligentController => 0x0E,
            DeviceClass::SatelliteCommunicationController => 0x0F,
            DeviceClass::EncryptionController => 0x10,
            DeviceClass::SignalProcessingController => 0x11,
            DeviceClass::ProcessingAccelerator => 0x12,
            DeviceClass::NonEssentialInstrumentation => 0x13,
            DeviceClass::Coprocessor => 0x40,
            DeviceClass::Unassigned => 0xFF,
            DeviceClass::Unknown(other) => other,
        }
    }
}

impl super::PCIDeviceTree {
    pub fn from_pcie(spaces: Vec<PCIeConfigSpace>) -> Self {
        let root_bus = parse_bus(spaces.as_slice(), &PCIeBus::new(0));
        Self {
            root: super::PCIDeviceTreeNode::Bus(root_bus),
        }
    }
}

fn get_bus_base(spaces: &[PCIeConfigSpace], bus: &PCIeBus) -> VirtAddr {
    for space in spaces {
        if !space.contains_bus(bus.bus) {
            continue;
        }
        let offset = (bus.bus - *space.buses.start()) as u64 * BUS_SIZE;
        return space.virt_base + offset;
    }

    panic!("PCI: bus {} not found", bus.bus);
}

fn parse_bus(spaces: &[PCIeConfigSpace], bus: &PCIeBus) -> super::PCIBusDevice {
    let base = get_bus_base(spaces, bus);
    let mut devices = Vec::new();
    for i in 0u64..32u64 {
        let mut device = super::PCIDevice::empty(i as u8);
        let device_addr = base + i * DEVICE_SIZE;
        // Addr of device = addr of function 0
        let header_type = PCIeDevice::new(device_addr).read::<u8>(PCIeFunctionOffset::HeaderType);
        // Optimization - Branchless version for:
        // if (header_type & 0x80) == 0 { 1 } else { 8 }
        let functions = ((header_type >> 7) & 1) as u64 * 7 + 1;
        for i in 0..functions {
            let function = PCIeDevice::new(device_addr + i * 4096);
            let vendor = function.read::<u16>(PCIeFunctionOffset::VendorID);
            // 0xFFFF means that the function is not present
            if vendor == 0xFFFF {
                continue;
            }
            let device_id = function.read::<u16>(PCIeFunctionOffset::DeviceID);
            let revision = function.read::<u8>(PCIeFunctionOffset::RevisionID);
            let class = function.class();
            let subclass = function.subclass();
            let subvendor = function.subvendor();
            let subdevice = function.subsystem();

            if class == DeviceClass::BridgeDevice && subclass == 0x04 {
                log::warn!("PCI: found PCI-to-PCI bridge");
            }

            device.functions.push((
                super::PCIDev {
                    bars: function.get_bars(),
                    revision,
                    dev: crate::Device {
                        allocator: crate::DeviceAllocator {},
                        driver_data: None,
                    },
                },
                super::PCIDeviceId {
                    vendor,
                    device: device_id,
                    subvendor,
                    subdevice,
                    class,
                    subclass,
                    ..Default::default()
                },
            ));
        }
        if !device.functions.is_empty() {
            devices.push(super::PCIDeviceTreeNode::Device(device));
        }
    }
    super::PCIBusDevice {
        bus_number: bus.bus,
        devices,
    }
}
