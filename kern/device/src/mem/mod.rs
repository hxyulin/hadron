//! Memory management

use x86_64::{PhysAddr, VirtAddr};

/// A memory mapped region
#[derive(Debug, Clone, Copy)]
pub struct MMRegion {
    base_phys: PhysAddr,
    base_virt: VirtAddr,
    size: usize,
}
