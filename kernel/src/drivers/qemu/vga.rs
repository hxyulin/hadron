//! QEMU VGA driver
//!
//! Specification: [QEMU VGA Device](https://www.qemu.org/docs/master/specs/standard-vga.html)

use spin::Mutex;
use x86_64::VirtAddr;

use crate::{base::{io::mmio::OffsetMmio, mem::Arc, pci::PCIeDeviceInfo}, drivers::Driver};

pub struct Vga {

}

struct MmioArea {
    /// VGA Input Index Port (0x3C0)
    input_index: OffsetMmio<u8, 0x0400>,
    /// VGA Input Data Port (0x3C1)
    input_data: OffsetMmio<u8, 0x0401>,
    /// VGA Output Port (0x3C2)
    output: OffsetMmio<u8, 0x0402>,

}

impl Vga {
    pub fn new(_info: PCIeDeviceInfo) -> Self {
        Self {}
    }

    pub fn create(info: PCIeDeviceInfo) -> Arc<Mutex<dyn Driver>> {
        Arc::new(Mutex::new(Self::new(info)))
    }
}

impl Driver for Vga {}
