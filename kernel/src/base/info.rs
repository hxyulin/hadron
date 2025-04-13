//! The base info module.
//! This contains the base KernelInfo struct, which contains information about the kernel.

use alloc::{boxed::Box, vec::Vec};
use conquer_once::spin::OnceCell;
use spin::{Mutex, RwLock};

use crate::util::timer::Timer;

use super::{arch::x86_64::apic::Apics, io::mmio::KernelMmio, pci::PCIeDeviceInfo};

pub struct KernelInfo {
    pub mmio: Mutex<KernelMmio>,
    pub pics: OnceCell<Mutex<Apics>>,
    pub timer: OnceCell<RwLock<Box<dyn Timer>>>,
    /// The PCI devices
    ///
    /// This stores a list of devices, and whether or not a driver has been found for the device
    pub pci_devices: RwLock<Vec<(PCIeDeviceInfo, bool)>>,
}

impl KernelInfo {
    pub const fn new() -> Self {
        Self {
            mmio: Mutex::new(KernelMmio::new()),
            pics: OnceCell::uninit(),
            timer: OnceCell::uninit(),
            pci_devices: RwLock::new(Vec::new()),
        }
    }
}

static KERNEL_INFO: KernelInfo = KernelInfo::new();

#[inline]
#[allow(static_mut_refs)]
pub fn kernel_info() -> &'static KernelInfo {
    &KERNEL_INFO
}
