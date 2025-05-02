//! The VGA driver for Bochs / QEMU

use hadron_driver_api::{
    DeviceClass,
    pci::{PCIDev, PCIDeviceId, PCIDriver},
};

#[used]
#[unsafe(link_section = ".pci_drivers")]
static DRIVER: PCIDriver = PCIDriver {
    name: "Bochs VGA",
    id_table: &[PCIDeviceId {
        vendor: 0x1234,
        device: 0x5678,
        // REDHAT / QEMU
        subvendor: 0x1af4,
        // QEMU
        subdevice: 0x1100,
        class: DeviceClass::DisplayController,
        subclass: 0,
        data: "qemu".as_bytes(),
    }],
    probe,
};

fn probe(_pci_dev: &mut PCIDev, _pci_id: &PCIDeviceId) -> u32 {
    0
}
