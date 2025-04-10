use x86_64::VirtAddr;
// The page tables are stored like this:
// 1. PML4 (0xFFFF_8000_0000_0000)
// 2. PDPT0-511
// 2. PD0-262143
// 2. PT0-134217727

/// This is recursive index 510
pub const PAGE_TABLES_START: VirtAddr = VirtAddr::new(0xFFFF_FF00_0000_0000);
pub const PAGE_TABLES_SIZE: u64 = 0xFFFF_FF80_0000_0000 - PAGE_TABLES_START.as_u64();
pub const KERNEL_TEXT: VirtAddr = VirtAddr::new(0xFFFF_FFFF_8000_0000);
pub const KERNEL_TEXT_SIZE: u64 = 0xFFFF_FFFF_8FFF_FFFF - KERNEL_TEXT.as_u64();
pub const KERNEL_STACK_START: VirtAddr = VirtAddr::new(0xFFFF_FFFF_9000_0000);
pub const KERNEL_STACK_SIZE: u64 = 0xFFFF_FFFF_9FFF_FFFF - KERNEL_STACK_START.as_u64();
pub const KERNEL_HEAP: VirtAddr = VirtAddr::new(0xFFFF_FFFF_A000_0000);
pub const KERNEL_HEAP_SIZE: u64 = 0xFFFF_FFFF_BFFF_FFFF - KERNEL_HEAP.as_u64();
pub const FRAMEBUFFER: VirtAddr = VirtAddr::new(0xFFFF_FFFF_C000_0000);
pub const FRAMEBUFFER_SIZE: u64 = 0xFFFF_FFFF_DFFF_FFFF - FRAMEBUFFER.as_u64();
pub const ACPI_TABLES: VirtAddr = VirtAddr::new(0xFFFF_FFFF_E000_0000);
pub const ACPI_TABLES_SIZE: u64 = 0xFFFF_FFFF_EFFF_FFFF - ACPI_TABLES.as_u64();
pub const MMIO_SPACE: VirtAddr = VirtAddr::new(0xFFFF_FFFF_0000_0000);
pub const MMIO_SPACE_SIZE: u64 = 0xFFFF_FFFF_FFFF_FFFF - MMIO_SPACE.as_u64();
