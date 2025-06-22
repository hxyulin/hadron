use linked_list_allocator::LockedHeap;
use x86_64::{PhysAddr, VirtAddr};

use super::arch::memory_map::BootstrapMemoryMap;
use hadron_base::base::mem::sync::RacyCell;

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

/// # Safety
///
/// This function is only safe to call if there are no other references to the boot info.
#[inline]
#[allow(static_mut_refs)]
pub(super) unsafe fn boot_info_mut() -> &'static mut BootInfo {
    BOOT_INFO.get_mut()
}
