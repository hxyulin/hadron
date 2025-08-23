use core::ops::{Index, IndexMut};

use crate::{
    arch::{PhysAddr, VirtAddr},
    mm::{
        mappings,
        paging::{FrameAllocator, PageSize, PhysFrame, Size4KiB},
    },
};

#[repr(C, align(4096))]
#[derive(Debug)]
pub struct PageTable {
    pub entries: [PageTableEntry; 512],
}

impl PageTable {
    pub const fn new() -> Self {
        Self {
            entries: [const { PageTableEntry::new() }; 512],
        }
    }
}

impl Index<usize> for PageTable {
    type Output = PageTableEntry;

    fn index(&self, index: usize) -> &Self::Output {
        &self.entries[index]
    }
}

impl IndexMut<usize> for PageTable {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.entries[index]
    }
}

#[repr(transparent)]
#[derive(Debug)]
pub struct PageTableEntry {
    entry: u64
}

impl PageTableEntry {
    const PHYS_ADDR_MASK: u64 = 0x000f_ffff_ffff_f000u64;

    pub const fn new() -> Self {
        Self {
            entry: 0,
        }
    }

    pub fn addr(&self) -> PhysAddr {
        PhysAddr::new((self.entry & Self::PHYS_ADDR_MASK) as usize)
    }

    pub fn set_addr(&mut self, addr: PhysAddr, flags: PageTableFlags) {
        assert!(addr.is_aligned(Size4KiB::SIZE), "page address is not page aligned");
        self.entry |= addr.as_u64() | flags.bits();
    }

    pub fn flags(&self) -> PageTableFlags {
        PageTableFlags::from_bits_truncate(self.entry & !Self::PHYS_ADDR_MASK)
    }

    pub fn set_flags(&mut self, flags: PageTableFlags) {
        self.entry = self.addr().as_u64() | flags.bits();
    }

    pub fn set_frame(&mut self, frame: PhysFrame, flags: PageTableFlags) {
        self.entry = frame.start_address().as_u64() | flags.bits()
    }
}

/// THe Page Table for the Kernel
///
/// The Page Table is implemented using the Direct Mapping Strategy
pub struct KernelPageTable {
    pml4: VirtAddr,
}

impl KernelPageTable {
    pub const DIRECT_MAP_OFFSET: usize = mappings::PAGE_TABLE_START.as_usize();

    pub fn new(pml4: VirtAddr) -> Self {
        Self { pml4 }
    }
}

bitflags::bitflags! {
    pub struct PageTableFlags: u64 {
        const PRESENT = 1 << 0;
        const WRITABLE = 1 << 1;
        const USER = 1 << 2;
        const WRITE_THROUGH = 1 << 3;
        const NO_CACHE = 1 << 4;
        const ACCESSED = 1 << 5;
        const DIRTY = 1 << 6;
        const HUGE_PAGE = 1 << 7;
        const GLOBAL = 1 << 8;
        const BIT_9 = 1 << 9;
        const BIT_10 = 1 << 10;
        const BIT_11 = 1 << 11;
        const BIT_52 = 1 << 52;
        const BIT_53 = 1 << 53;
        const BIT_54 = 1 << 54;
        const BIT_55 = 1 << 55;
        const BIT_56 = 1 << 56;
        const BIT_57 = 1 << 57;
        const BIT_58 = 1 << 58;
        const BIT_59 = 1 << 59;
        const BIT_60 = 1 << 60;
        const PKEY_D = 1 << 61;
        const PKEY_U = 1 << 62;
        const NO_EXECUTE = 1 << 63;
    }
}

pub trait Mapper<S: PageSize> {
    fn map_page(frame: PhysFrame, allocator: impl FrameAllocator<S>, options: PageTableFlags) -> Result<Flush, ()>;
}

pub struct Flush {
    addr: VirtAddr,
}

impl Flush {
    pub fn flush(self) {
        // SAFETY: The address is a valid and mapped page
        unsafe { crate::arch::instructions::invlpg(self.addr) };
    }
}
