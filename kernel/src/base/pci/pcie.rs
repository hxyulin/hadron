use core::ops::DerefMut;

use alloc::vec::Vec;
use x86_64::{
    PhysAddr, VirtAddr,
    structures::paging::{Page, PageSize, PageTableFlags, PhysFrame, Size2MiB},
};

use crate::base::mem::page_table::PageTable;

#[derive(Debug, Clone)]
pub struct PCIeDeviceInfo {
    pub addr: PhysAddr,
    pub vendor_id: u16,
    pub device_id: u16,
    pub class: PCIDeviceClass,
    pub capabilities: PCICapabilities,
    pub bars: [u32; 6],
}

impl PCIeDeviceInfo {
    /// Returns the address of the PCIe device
    ///
    /// # Safety
    /// This function is unsafe because the caller must ensure that the
    /// address in the McfgEntry is valid.
    pub unsafe fn from_mcfg(entry: &acpi::mcfg::McfgEntry) -> Vec<Self> {
        let mut devices = Vec::new();
        const FRAMES_PER_BUS: u64 = 8 * 32;
        let bus_count = (entry.bus_number_end - entry.bus_number_start) as usize + 1;
        // Divide by 512 because we are using 2 MiB pages
        let pages_needed = bus_count as u64 * FRAMES_PER_BUS as u64 / 512;
        let mut buses = Vec::with_capacity(bus_count);
        for i in entry.bus_number_start..entry.bus_number_end {
            buses.push(PCIEBus::new(i));
        }
        log::debug!("PCI: mapping config space ({} 2MiB pages)", pages_needed);
        {
            // TODO: We can probably use bigger frames here like 2MiB
            let mut page_table = crate::base::mem::PAGE_TABLE.lock();
            let mut allocator = crate::base::mem::FRAME_ALLOCATOR.lock();
            for i in 0..pages_needed {
                let addr = entry.base_address + i * Size2MiB::SIZE;
                unsafe {
                    page_table.map_with_allocator(
                        Page::<Size2MiB>::from_start_address(VirtAddr::new(addr)).unwrap(),
                        PhysFrame::<Size2MiB>::from_start_address(PhysAddr::new(addr)).unwrap(),
                        PageTableFlags::PRESENT
                            | PageTableFlags::WRITABLE
                            | PageTableFlags::NO_EXECUTE
                            | PageTableFlags::NO_CACHE,
                        allocator.deref_mut(),
                    )
                };
            }
        }

        let pci_base = VirtAddr::new(entry.base_address);
        for bus in &buses {
            for device in &bus.devices() {
                for function in &device.functions() {
                    let vendor_id = function.read::<u16>(pci_base, PCIEFunctionOffset::VendorID);
                    // Vendor ID 0xFFFF means that the function is not present
                    if vendor_id == 0xFFFF {
                        continue;
                    }
                    devices.push(PCIeDeviceInfo {
                        addr: PhysAddr::new(function.get_mmio_addr(pci_base, 0).as_u64()),
                        vendor_id,
                        device_id: function.read(pci_base, PCIEFunctionOffset::DeviceID),
                        class: function.device_class(pci_base),
                        capabilities: function.capabilities(pci_base),
                        bars: function.get_bars(pci_base),
                    });
                }
            }
        }

        {
            log::debug!("PCI: unmapping config space");
            // Unmap the PCI config space
            let mut page_table = crate::base::mem::PAGE_TABLE.lock();
            for i in 0..pages_needed {
                let addr = entry.base_address + i * Size2MiB::SIZE;
                unsafe { page_table.unmap(Page::<Size2MiB>::from_start_address(VirtAddr::new(addr)).unwrap()) };
            }
        }

        devices
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PCIEBus {
    bus_number: u8,
}

impl PCIEBus {
    fn new(bus_number: u8) -> Self {
        Self { bus_number }
    }

    pub fn devices(&self) -> [PCIEDevice; 32] {
        let mut devices = [PCIEDevice::new(*self, 0); 32];
        for (i, device) in devices.iter_mut().enumerate() {
            device.device = i as u8;
        }
        devices
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PCIEDevice {
    bus: PCIEBus,
    device: u8,
}

impl PCIEDevice {
    fn new(bus: PCIEBus, device: u8) -> Self {
        Self { bus, device }
    }

    pub fn functions(&self) -> [PCIEFunction; 8] {
        let mut functions = [PCIEFunction::new(*self, 0); 8];
        for (i, function) in functions.iter_mut().enumerate() {
            function.function = i as u8;
        }
        functions
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

#[derive(Debug, Clone, Copy)]
pub struct PCIEFunction {
    device: PCIEDevice,
    function: u8,
}

impl PCIEFunction {
    fn new(device: PCIEDevice, function: u8) -> Self {
        Self { device, function }
    }

    fn get_mmio_addr(&self, base: VirtAddr, offset: u16) -> VirtAddr {
        base + ((self.device.bus.bus_number as u64) << 20)
            + ((self.device.device as u64) << 15)
            + ((self.function as u64) << 12)
            + offset as u64
    }

    pub fn read<T>(&self, base: VirtAddr, offset: PCIEFunctionOffset) -> T {
        unsafe { self.read_internal::<T>(base, offset as u16) }
    }

    pub fn write<T>(&self, base: VirtAddr, offset: PCIEFunctionOffset, value: T) {
        unsafe { self.write_internal::<T>(base, offset as u16, value) };
    }

    unsafe fn read_internal<T>(&self, base: VirtAddr, offset: u16) -> T {
        unsafe { self.get_mmio_addr(base, offset).as_ptr::<T>().read_volatile() }
    }

    unsafe fn write_internal<T>(&self, base: VirtAddr, offset: u16, value: T) {
        unsafe { self.get_mmio_addr(base, offset).as_mut_ptr::<T>().write_volatile(value) }
    }

    pub fn device_class(&self, base: VirtAddr) -> PCIDeviceClass {
        let class = self.read::<u8>(base, PCIEFunctionOffset::Class);
        PCIDeviceClass::from_u8(class)
    }

    pub fn capabilities(&self, base: VirtAddr) -> PCICapabilities {
        let mut capabilities = PCICapabilities::empty();

        let mut current_offset = self.read::<u16>(base, PCIEFunctionOffset::CapabilitiesPointer);

        while current_offset != 0 {
            // Read the Capability ID and the Next Capability Pointer
            let cap_id = unsafe { self.read_internal::<u32>(base, current_offset) & 0xFF };
            let next_pointer = unsafe { (self.read_internal::<u32>(base, current_offset) >> 8) & 0xFF };

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

    pub fn get_bars(&self, base: VirtAddr) -> [u32; 6] {
        let mut bars = [0; 6];
        for i in 0..6 {
            // SAFETY: The offset is valid because it is within the range of the function
            let addr = unsafe { self.read_internal::<u32>(base, PCIEFunctionOffset::Bar0 as u16 + i * 4) };
            bars[i as usize] = addr;
        }
        bars
    }
}

#[repr(u16)]
#[derive(Debug, Clone, Copy)]
pub enum PCIEFunctionOffset {
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

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum PCIDeviceClass {
    Unclassified = 0x00,
    MassStorageController = 0x01,
    NetworkController = 0x02,
    DisplayController = 0x03,
    MultimediaController = 0x04,
    MemoryController = 0x05,
    BridgeDevice = 0x06,
    SimpleCommunicationController = 0x07,
    BaseSystemPeripheral = 0x08,
    InputDeviceController = 0x09,
    DockingStation = 0x0A,
    Processor = 0x0B,
    SerialBusController = 0x0C,
    WirelessController = 0x0D,
    IntelligentController = 0x0E,
    SatelliteCommunicationController = 0x0F,
    EncryptionController = 0x10,
    SignalProcessingController = 0x11,
    ProcessingAccelerator = 0x12,
    NonEssentialInstrumentation = 0x13,
    Coprocessor = 0x40,
    Unassigned = 0xFF,
}

impl PCIDeviceClass {
    pub fn from_u8(value: u8) -> PCIDeviceClass {
        match value {
            0x00 => PCIDeviceClass::Unclassified,
            0x01 => PCIDeviceClass::MassStorageController,
            0x02 => PCIDeviceClass::NetworkController,
            0x03 => PCIDeviceClass::DisplayController,
            0x04 => PCIDeviceClass::MultimediaController,
            0x05 => PCIDeviceClass::MemoryController,
            0x06 => PCIDeviceClass::BridgeDevice,
            0x07 => PCIDeviceClass::SimpleCommunicationController,
            0x08 => PCIDeviceClass::BaseSystemPeripheral,
            0x09 => PCIDeviceClass::InputDeviceController,
            0x0A => PCIDeviceClass::DockingStation,
            0x0B => PCIDeviceClass::Processor,
            0x0C => PCIDeviceClass::SerialBusController,
            0x0D => PCIDeviceClass::WirelessController,
            0x0E => PCIDeviceClass::IntelligentController,
            0x0F => PCIDeviceClass::SatelliteCommunicationController,
            0x10 => PCIDeviceClass::EncryptionController,
            0x11 => PCIDeviceClass::SignalProcessingController,
            0x12 => PCIDeviceClass::ProcessingAccelerator,
            0x13 => PCIDeviceClass::NonEssentialInstrumentation,
            0x40 => PCIDeviceClass::Coprocessor,
            0xFF => PCIDeviceClass::Unassigned,
            _ => PCIDeviceClass::Unclassified,
        }
    }

    pub fn to_u8(self) -> u8 {
        self as u8
    }
}
