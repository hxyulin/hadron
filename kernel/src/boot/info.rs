use linked_list_allocator::LockedHeap;
use x86_64::{PhysAddr, VirtAddr};

use super::arch::memory_map::BootstrapMemoryMap;
use crate::base::mem::sync::RacyCell;

pub struct BootInfo {
    pub hhdm_offset: u64,
    pub kernel_start_phys: PhysAddr,
    pub kernel_start_virt: VirtAddr,
    pub rsdp_addr: PhysAddr,
    pub memory_map: BootstrapMemoryMap,
    pub heap: (VirtAddr, u64),
    pub allocator: LockedHeap,
}

impl BootInfo {
    pub const fn uninit() -> Self {
        Self {
            hhdm_offset: 0,
            kernel_start_phys: PhysAddr::new(0),
            kernel_start_virt: VirtAddr::new(0),
            rsdp_addr: PhysAddr::new(0),
            memory_map: BootstrapMemoryMap::empty(),
            heap: (VirtAddr::new(0), 0),
            allocator: LockedHeap::empty(),
        }
    }
}

static BOOT_INFO: RacyCell<BootInfo> = RacyCell::new(BootInfo::uninit());

#[inline]
#[allow(static_mut_refs)]
pub(super) fn boot_info() -> &'static BootInfo {
    BOOT_INFO.get()
}

/// # Safety
///
/// This function is only safe to call if there are no other references to the boot info.
#[inline]
#[allow(static_mut_refs)]
pub(super) unsafe fn boot_info_mut() -> &'static mut BootInfo {
    BOOT_INFO.get_mut()
}

#[inline]
pub(super) fn with_boot_info_mut<T, F: FnOnce(&mut BootInfo) -> T>(f: F) -> T {
    static LOCK: spin::Mutex<()> = spin::Mutex::new(());
    let _guard = LOCK.try_lock().expect("Kernel info is already borrowed");
    let boot_info = unsafe { boot_info_mut() };
    f(boot_info)
}
