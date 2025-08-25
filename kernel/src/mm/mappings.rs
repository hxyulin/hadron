//! Memory Mappings for the Kernel

use crate::arch::VirtAddr;

/// Each Kernel Process gets 64KiB
pub const KERNEL_STACK_SIZE: usize = 64 * 1024;

/// The Start of the User Memory (Lower Half)
pub const USER_MEM_START: VirtAddr = VirtAddr::new(0x0000_0000_0000_0000);
/// The Size of the User Memory
/// This is 128TiB (all of the lower half)
pub const USER_MEM_SIZE: usize = 0x0000_8000_0000_0000;

/// The Start of the Kernel Memory (Higher Half)
pub const KERNEL_MEM_START: VirtAddr = VirtAddr::new(0xFFFF_8000_0000_0000);
/// The Size of the Kernel Memory
/// This is 128TiB (all of the higher half)
pub const KERNEL_MEM_SIZE: usize = 0usize.wrapping_sub(KERNEL_MEM_START.as_usize());

pub const PAGE_TABLE_START: VirtAddr = VirtAddr::new(0xFFFF_8000_0000_0000);
pub const PAGE_TABLE_SIZE: usize = KERNEL_HEAP_START.as_usize() - PAGE_TABLE_START.as_usize();

pub const KERNEL_HEAP_START: VirtAddr = VirtAddr::new(0xFFFF_C000_0000_0000);
pub const KERNEL_HEAP_END: VirtAddr = VirtAddr::new(0xFFFF_C010_0000_0000);
/// The Size of the Kernel Heap (64 GiB)
pub const KERNEL_HEAP_SIZE: usize = KERNEL_HEAP_END.as_usize() - KERNEL_HEAP_START.as_usize();

pub const KERNEL_STACK_START: VirtAddr = VirtAddr::new(0xFFFF_C020_0000_0000);
pub const KERNEL_STACK_END: VirtAddr = VirtAddr::new(0xFFFF_C020_8000_0000);
/// The Size of the Kernel Stack (Maximum Size 2GiB)
pub const TOTAL_KERNEL_STACK_SIZE: usize = KERNEL_STACK_END.as_usize() - KERNEL_STACK_START.as_usize();

pub const FRAMEBUFFER_START: VirtAddr = VirtAddr::new(0xFFFF_D000_0000_0000);
pub const FRAMEBUFFER_END: VirtAddr = VirtAddr::new(0xFFFF_E000_0000_0000);
/// The Size of the Framebuffer (Currently 16 TiB)
pub const FRAMEBUFFER_SIZE: usize = FRAMEBUFFER_END.as_usize() - FRAMEBUFFER_START.as_usize();

pub const MMIO_SPACE_START: VirtAddr = VirtAddr::new(0xFFFF_E000_8000_0000);
pub const MMIO_SPACE_END: VirtAddr = VirtAddr::new(0xFFFF_F000_8000_0000);
/// The Size of the MMIO Space (Currently 16 TiB)
pub const MMIO_SPACE_SIZE: usize = MMIO_SPACE_END.as_usize() - MMIO_SPACE_START.as_usize();

pub const MEMORY_MAPPINGS: VirtAddr = VirtAddr::new(0xFFFF_F800_0000_0000);
pub const MEMORY_MAPPINGS_SIZE: usize = 0xFFFF_F900_0000_0000 - MEMORY_MAPPINGS.as_usize();

pub const KERNEL_TEXT_START: VirtAddr = VirtAddr::new(0xFFFF_FFFF_8000_0000);
pub const KERNEL_TEXT_SIZE: usize = 0usize.wrapping_sub(KERNEL_TEXT_START.as_usize());
