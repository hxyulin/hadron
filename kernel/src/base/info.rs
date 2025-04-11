//! The base info module.
//! This contains the base KernelInfo struct, which contains information about the kernel.

use spin::Mutex;

use crate::{boot::info::BootInfo, devices::DeviceManager};

use super::mem::{frame_allocator::KernelFrameAllocator, page_table::KernelPageTable};

pub struct RuntimeInfo {
    pub(super) frame_allocator: Mutex<KernelFrameAllocator>,
    pub(super) page_table: Mutex<KernelPageTable>,
    pub devices: DeviceManager,
}

impl RuntimeInfo {
    pub fn new(frame_allocator: Mutex<KernelFrameAllocator>, page_table: Mutex<KernelPageTable>) -> Self {
        Self {
            frame_allocator,
            page_table,
            devices: DeviceManager::new(),
        }
    }
}

#[allow(clippy::large_enum_variant)]
pub enum KernelInfo {
    Boot(BootInfo),
    Kernel(RuntimeInfo),
}

pub(crate) static mut KERNEL_INFO: KernelInfo = KernelInfo::Boot(BootInfo::default());

#[inline]
#[allow(static_mut_refs)]
pub fn try_kernel_info() -> Option<&'static RuntimeInfo> {
    match unsafe { &KERNEL_INFO } {
        KernelInfo::Kernel(runtime_info) => Some(runtime_info),
        _ => None,
    }
}

#[inline]
#[allow(static_mut_refs)]
pub fn kernel_info() -> &'static RuntimeInfo {
    debug_assert!(
        matches!(unsafe { &KERNEL_INFO }, KernelInfo::Kernel(_)),
        "Invalid kernel info"
    );
    match unsafe { &KERNEL_INFO } {
        KernelInfo::Kernel(runtime_info) => runtime_info,
        _ => unsafe {
            core::hint::unreachable_unchecked();
        },
    }
}
