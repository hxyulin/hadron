use crate::{arch::{PhysAddr, VirtAddr}, boot::memory_map::BootstrapMemoryMap, sync::cell::RacyCell};
use crate::dev::drivers::platform::fb::FramebufferInfoAddr;

pub struct BootInfo {
    pub hhdm_offset: u64,
    pub kernel_phys: PhysAddr,
    pub kernel_virt: VirtAddr,
    pub memory_map: BootstrapMemoryMap,
    pub rsdp_addr: PhysAddr,
    pub heap: (VirtAddr, usize),
    pub framebuffer: FramebufferInfoAddr,
}

impl BootInfo {
    pub const fn empty() -> Self {
        Self {
            hhdm_offset: 0,
            kernel_phys: PhysAddr::NULL,
            kernel_virt: VirtAddr::NULL,
            memory_map: BootstrapMemoryMap::empty(),
            rsdp_addr: PhysAddr::NULL,
            heap: (VirtAddr::NULL, 0),
            framebuffer: FramebufferInfoAddr::default(),
        }
    }
}

pub(super) static BOOT_INFO: RacyCell<BootInfo> = RacyCell::new(BootInfo::empty());
