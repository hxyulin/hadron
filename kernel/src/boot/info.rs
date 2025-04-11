use x86_64::{PhysAddr, VirtAddr};

use super::{
    arch::memory_map::MemoryMap,
    drivers::{framebuffer::FramebufferWriter, serial::SerialWriter},
};
use crate::base::info::{KERNEL_INFO, KernelInfo};

pub struct BootInfo {
    pub serial: SerialWriter,
    pub framebuffer: Option<FramebufferWriter>,

    pub hhdm_offset: u64,
    pub kernel_start_phys: PhysAddr,
    pub kernel_start_virt: VirtAddr,
    pub rsdp_addr: PhysAddr,
    pub memory_map: MemoryMap,
    pub heap: (VirtAddr, u64),
}

impl BootInfo {
    pub const fn default() -> Self {
        Self {
            serial: SerialWriter::new(0x3F8),
            framebuffer: None,

            hhdm_offset: 0,
            kernel_start_phys: PhysAddr::new(0),
            kernel_start_virt: VirtAddr::new(0),
            rsdp_addr: PhysAddr::new(0),
            memory_map: MemoryMap::default(),
            heap: (VirtAddr::new(0), 0),
        }
    }
}

#[inline]
#[allow(static_mut_refs)]
pub(super) fn boot_info() -> &'static BootInfo {
    debug_assert!(
        matches!(unsafe { &KERNEL_INFO }, KernelInfo::Boot(_)),
        "Invalid kernel info"
    );
    match unsafe { &KERNEL_INFO } {
        KernelInfo::Boot(boot_info) => boot_info,
        _ => unsafe {
            core::hint::unreachable_unchecked();
        },
    }
}

/// # Safety
///
/// This function is only safe to call if there are no other references to the boot info.
#[inline]
#[allow(static_mut_refs)]
pub(super) unsafe fn boot_info_mut() -> &'static mut BootInfo {
    debug_assert!(
        matches!(unsafe { &KERNEL_INFO }, KernelInfo::Boot(_)),
        "Invalid kernel info"
    );
    match unsafe { &mut KERNEL_INFO } {
        KernelInfo::Boot(boot_info) => boot_info,
        _ => unsafe {
            core::hint::unreachable_unchecked();
        },
    }
}

#[inline]
pub(super) fn with_boot_info_mut<T, F: FnOnce(&mut BootInfo) -> T>(f: F) -> T {
    static LOCK: spin::Mutex<()> = spin::Mutex::new(());
    let _guard = LOCK.try_lock().expect("Kernel info is already borrowed");
    let boot_info = unsafe { boot_info_mut() };
    f(boot_info)
}
