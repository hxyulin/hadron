use super::page_table::PageTable;
use crate::{
    base::mem::{mappings, sync::Arc},
    boot::arch::memory_map::{BootstrapMemoryMap, FrameBasedAllocator, MemoryRegionType},
};
use alloc::vec::Vec;
use x86_64::{
    PhysAddr,
    structures::paging::{FrameAllocator, Page, PageSize, PageTableFlags, Size4KiB},
};

use super::page_table::KernelPageTable;

#[derive(Debug)]
pub struct MemoryMap {
    pub(super) alloc: Arc<FrameBasedAllocator>,
    pub(super) entries: Vec<MemoryRegion, Arc<FrameBasedAllocator>>,
    pub(super) special: Vec<SpecialMemoryRegion>,
}

#[derive(Debug, Clone)]
pub struct MemoryRegion {
    pub(super) base: PhysAddr,
    pub(super) bitmap: Bitmap<Arc<FrameBasedAllocator>>,
}

#[derive(Clone)]
pub struct Bitmap<A: alloc::alloc::Allocator = alloc::alloc::Global>(Vec<u64, A>, usize);

impl MemoryMap {
    /// Parses the memory map from the bootstrap info
    ///
    /// This will allocate at most [`Self::MAX_BOOTSTRAP_PAGES`] pages for the kernel heap,
    /// storing the rest of the usable memory in the `special` field
    pub fn from_bootstrap(memory_map: &mut BootstrapMemoryMap, page_table: &mut KernelPageTable) -> Self {
        use crate::boot::arch::x86_64::frame_allocator::BasicFrameAllocator;

        /// The number of entries we need to reserve for the memory map, for deallocation of the
        /// bootstrap structures
        const RESERVED_ENTRIES: u64 = 8;
        // We need to calculate how much memory we need for the entire memory map
        let mut required_size = size_of::<MemoryRegion>() as u64 * RESERVED_ENTRIES;
        for region in memory_map.iter() {
            required_size += size_of::<MemoryRegion>() as u64;
            let bitmap_size = region.length.div_ceil(32768);
            // We align it to 64 bytes to be conservative
            required_size += (bitmap_size + 63) & !63;
        }

        let mut frame_allocator = BasicFrameAllocator::new(memory_map);
        let required_frames = required_size.div_ceil(Size4KiB::SIZE);
        for i in 0..required_frames {
            let addr = mappings::MEMORY_MAPPINGS + i * Size4KiB::SIZE;
            unsafe {
                page_table.map_with_allocator(
                    Page::from_start_address(addr).unwrap(),
                    frame_allocator.allocate_frame().unwrap(),
                    PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_EXECUTE,
                    &mut frame_allocator,
                )
            };
        }

        let alloc = Arc::new(unsafe {
            FrameBasedAllocator::new(mappings::MEMORY_MAPPINGS, (required_frames * Size4KiB::SIZE) as usize)
        });
        let mut entries = Vec::new_in(alloc.clone());
        entries.reserve_exact(memory_map.len());
        for entry in memory_map
            .iter()
            .filter(|entry| entry.ty() == MemoryRegionType::Usable && entry.length() > 0)
        {
            entries.push(MemoryRegion::from_base_and_length(
                entry.base(),
                entry.length(),
                alloc.clone(),
            ));
        }

        Self {
            alloc,
            entries,
            special: Vec::new(),
        }
    }

    pub fn push_entry(&mut self, entry: crate::boot::arch::memory_map::MemoryMapEntry) {
        self.entries.push(MemoryRegion::from_base_and_length(
            entry.base(),
            entry.length(),
            self.alloc.clone(),
        ));
    }
}

impl MemoryRegion {
    pub fn from_base_and_length(base: PhysAddr, length: u64, alloc: Arc<FrameBasedAllocator>) -> Self {
        let pages = length / Size4KiB::SIZE;
        let bitmap = Bitmap::new_in(pages as usize, alloc);
        Self { base, bitmap }
    }

    pub fn pages(&self) -> u64 {
        self.bitmap.size() as u64
    }

    pub(super) fn contains(&self, addr: PhysAddr) -> bool {
        addr >= self.base && addr < self.base + self.pages() * Size4KiB::SIZE
    }

    pub(super) fn allocate(&mut self) -> Option<usize> {
        let idx = self.bitmap.find_free()?;
        self.bitmap.set(idx, true);
        Some(idx)
    }

    pub(super) fn deallocate(&mut self, idx: usize) {
        self.bitmap.set(idx, false);
    }

    /// Resizes the bitmap to the given size.
    ///
    /// # Safety
    /// This function is unsafe because the new size must be valid (usable) memory.
    unsafe fn resize(&mut self, new_size_pages: usize) {
        unsafe { self.bitmap.resize(new_size_pages) };
    }
}

impl Bitmap<alloc::alloc::Global> {
    pub fn new(size: usize) -> Self {
        Self::new_in(size, alloc::alloc::Global)
    }
}

impl<A> Bitmap<A>
where
    A: alloc::alloc::Allocator,
{
    pub fn new_in(size: usize, alloc: A) -> Self {
        let len = size.div_ceil(64);
        let mut data = Vec::new_in(alloc);
        data.reserve_exact(len);
        data.resize(len, 0);
        Self(data, size)
    }

    pub fn size(&self) -> usize {
        self.1
    }

    pub fn set(&mut self, idx: usize, value: bool) {
        let bit = idx % 64;
        let idx = idx / 64;
        assert!(idx < self.1);
        if value {
            self.0[idx] |= 1 << bit;
        } else {
            self.0[idx] &= !(1 << bit);
        }
    }

    pub fn get(&self, idx: usize) -> bool {
        let idx = idx / 64;
        let bit = idx % 64;
        let byte = idx / 8;
        (self.0[byte] & (1 << bit)) != 0
    }

    pub fn find_free(&self) -> Option<usize> {
        for (idx, byte) in self.0.iter().enumerate() {
            // We don't need to check bit-by-bit, because the bitmap is a full 64-bit word
            if *byte == u64::MAX {
                continue;
            }
            for bit in 0..64 {
                let idx = idx * 64 + bit;
                if idx >= self.1 {
                    return None;
                }

                if (byte & (1 << bit)) == 0 {
                    return Some(idx);
                }
            }
        }
        None
    }

    pub unsafe fn resize(&mut self, new_size: usize) {
        assert!(new_size > self.size());
        let new_size = new_size.div_ceil(64);
        self.0.resize(new_size, 0);
        self.1 = new_size;
    }
}

impl<A> core::fmt::Debug for Bitmap<A>
where
    A: alloc::alloc::Allocator,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("Bitmap")
            .field(&format_args!("{} bits", self.0.len() * 64))
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryRegionTag {
    /// Usable memory that is not allocated
    Unallocated,
    BootloaderReclaimable,
    KernelAndModules,
    Framebuffer,
    AcpiReclaimable,
    AcpiNvs,
}

#[derive(Debug, Clone, Copy)]
pub struct SpecialMemoryRegion {
    pub(crate) base: PhysAddr,
    pub(crate) length: u64,
    pub(crate) tag: MemoryRegionTag,
}

impl MemoryRegionTag {
    pub fn from_type(ty: MemoryRegionType) -> Option<Self> {
        match ty {
            MemoryRegionType::BootloaderReclaimable => Some(Self::BootloaderReclaimable),
            MemoryRegionType::KernelAndModules => Some(Self::KernelAndModules),
            MemoryRegionType::Framebuffer => Some(Self::Framebuffer),
            MemoryRegionType::AcpiReclaimable => Some(Self::AcpiReclaimable),
            MemoryRegionType::AcpiNvs => Some(Self::AcpiNvs),
            _ => None,
        }
    }
}
