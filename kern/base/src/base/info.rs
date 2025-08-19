//! The base info module.
//! This contains the base KernelInfo struct, which contains information about the kernel.

use conquer_once::spin::OnceCell;
use spin::Mutex;
use x86_64::VirtAddr;

use super::arch::x86_64::apic::Apics;

pub struct KernelInfo {
    pub base_addr: Mutex<VirtAddr>,
    pub pics: OnceCell<Mutex<Apics>>,
}

impl KernelInfo {
    pub const fn new() -> Self {
        Self {
            base_addr: Mutex::new(VirtAddr::new(0)),
            pics: OnceCell::uninit(),
        }
    }
}

static KERNEL_INFO: KernelInfo = KernelInfo::new();

#[inline]
#[allow(static_mut_refs)]
pub fn kernel_info() -> &'static KernelInfo {
    &KERNEL_INFO
}
