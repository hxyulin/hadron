use x86_64::VirtAddr;
// The page tables are stored like this:
// 1. PML4 (0xFFFF_8000_0000_0000)
// 2. PDPT0-511
// 2. PD0-262143
// 2. PT0-134217727

pub const PAGE_TABLES_START: VirtAddr = VirtAddr::new(0xFFFF_FF00_0000_0000);
pub const PAGE_TABLES_SIZE: u64 = 0xFFFF_FF80_0000_0000 - PAGE_TABLES_START.as_u64();
pub const KERNEL_STACK_START: VirtAddr = VirtAddr::new(0xFFFF_FFFE_1000_0000);
pub const KERNEL_STACK_SIZE: u64 = 0xFFFF_FFFE_2000_0000 - KERNEL_STACK_START.as_u64();
pub const KERNEL_STACK_END: VirtAddr = VirtAddr::new(KERNEL_STACK_START.as_u64() + KERNEL_STACK_SIZE);
pub const KERNEL_HEAP: VirtAddr = VirtAddr::new(0xFFFF_FFFE_2000_0000);
pub const KERNEL_HEAP_SIZE: u64 = 0xFFFF_FFFE_4000_0000 - KERNEL_HEAP.as_u64();
pub const FRAMEBUFFER: VirtAddr = VirtAddr::new(0xFFFF_FFFE_4000_0000);
pub const FRAMEBUFFER_SIZE: u64 = 0xFFFF_FFFE_6000_0000 - FRAMEBUFFER.as_u64();
pub const MEMORY_MAPPINGS: VirtAddr = VirtAddr::new(0xFFFF_FFFE_6000_0000);
pub const MEMORY_MAPPINGS_SIZE: u64 = 0xFFFF_FFFE_7000_0000 - MEMORY_MAPPINGS.as_u64();
pub const MMIO_SPACE: VirtAddr = VirtAddr::new(0xFFFF_FFFE_7000_0000);
pub const MMIO_SPACE_SIZE: u64 = 0xFFFF_FFFF_8000_0000 - MMIO_SPACE.as_u64();
/// Limine KASLR loads the kernel lowest at 0xFFFF_FFFF_8000_0000, up to 0xFFFF_FFFF_FFFF_FFFF
pub const KERNEL_TEXT_START: VirtAddr = VirtAddr::new(0xFFFF_FFFF_8000_0000);
pub const KERNEL_TEXT_END: VirtAddr = VirtAddr::new(0xFFFF_FFFF_FFFF_FFFF);
