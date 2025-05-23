use alloc::vec::Vec;
use hadron_base::base::mem::{
    allocator::FrameBasedAllocator,
    mappings,
    memory_map::{MemoryMap, MemoryRegion, MemoryRegionTag},
    page_table::{KernelPageTable, PageTable},
};
use x86_64::{
    PhysAddr, VirtAddr,
    structures::paging::{FrameAllocator, Page, PageSize, PageTableFlags, Size4KiB},
};

use hadron_base::base::mem::sync::Arc;

use super::x86_64::frame_allocator::BasicFrameAllocator;

#[repr(u64)]
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryRegionType {
    Usable = 0,
    Reserved = 1,
    AcpiReclaimable = 2,
    AcpiNvs = 3,
    BadMemory = 4,
    BootloaderReclaimable = 5,
    KernelAndModules = 6,
    Framebuffer = 7,

    Allocated = 0x100,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MemoryMapEntry {
    pub(crate) base: PhysAddr,
    pub(crate) length: u64,
    // We can make the memory type 8bit, and have 56 more for future use
    pub(crate) memory_type: MemoryRegionType,
}

impl MemoryMapEntry {
    pub const fn new(base: PhysAddr, length: u64, memory_type: MemoryRegionType) -> Self {
        Self {
            base,
            length,
            memory_type,
        }
    }

    pub fn base(&self) -> PhysAddr {
        self.base
    }

    pub fn length(&self) -> u64 {
        self.length
    }

    pub fn end(&self) -> PhysAddr {
        PhysAddr::new(self.base.as_u64() + self.length)
    }

    pub fn ty(&self) -> MemoryRegionType {
        self.memory_type
    }

    pub fn set_type(&mut self, ty: MemoryRegionType) {
        self.memory_type = ty;
    }
}

#[derive(Debug)]
pub struct BootstrapMemoryMap {
    pub entries: Vec<MemoryMapEntry, FrameBasedAllocator>,
}

impl BootstrapMemoryMap {
    pub const fn empty() -> Self {
        Self {
            entries: Vec::new_in(FrameBasedAllocator::empty()),
        }
    }

    pub fn total_size(&self) -> u64 {
        self.entries
            .iter()
            .filter_map(|e| {
                if e.ty() == MemoryRegionType::Usable {
                    Some(e.length())
                } else {
                    None
                }
            })
            .sum()
    }

    pub fn reclaim_bootloader_memory(&mut self) {
        for region in self.iter_mut() {
            if region.ty() == MemoryRegionType::BootloaderReclaimable {
                region.memory_type = MemoryRegionType::Usable;
            }
        }
    }
    pub fn iter(&self) -> core::slice::Iter<'_, MemoryMapEntry> {
        self.entries.iter()
    }

    pub fn iter_mut(&mut self) -> core::slice::IterMut<'_, MemoryMapEntry> {
        self.entries.iter_mut()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn push(&mut self, entry: MemoryMapEntry) {
        self.entries
            .push_within_capacity(entry)
            .expect("memory map: ran out of memory");
    }

    /// Returns the mapped range of the memory map
    pub fn mapped_range(&self) -> (VirtAddr, u64) {
        self.entries.allocator().mapped_range()
    }
}

pub trait MainMemoryMap {
    fn from_bootstrap(memory_map: &mut BootstrapMemoryMap, page_table: &mut KernelPageTable) -> Self;
    fn push_entry(&mut self, entry: crate::boot::arch::memory_map::MemoryMapEntry);
}

impl MainMemoryMap for MemoryMap {
    /// Parses the memory map from the bootstrap info
    ///
    /// This will allocate at most [`Self::MAX_BOOTSTRAP_PAGES`] pages for the kernel heap,
    /// storing the rest of the usable memory in the `special` field
    fn from_bootstrap(memory_map: &mut BootstrapMemoryMap, page_table: &mut KernelPageTable) -> Self {
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

    fn push_entry(&mut self, entry: crate::boot::arch::memory_map::MemoryMapEntry) {
        self.entries.push(MemoryRegion::from_base_and_length(
            entry.base(),
            entry.length(),
            self.alloc.clone(),
        ));
    }
}

impl TryInto<MemoryRegionTag> for MemoryRegionType {
    type Error = ();

    fn try_into(self) -> Result<MemoryRegionTag, Self::Error> {
        match self {
            MemoryRegionType::BootloaderReclaimable => Ok(MemoryRegionTag::BootloaderReclaimable),
            MemoryRegionType::KernelAndModules => Ok(MemoryRegionTag::KernelAndModules),
            MemoryRegionType::Framebuffer => Ok(MemoryRegionTag::Framebuffer),
            MemoryRegionType::AcpiReclaimable => Ok(MemoryRegionTag::AcpiReclaimable),
            MemoryRegionType::AcpiNvs => Ok(MemoryRegionTag::AcpiNvs),
            _ => Err(()),
        }
    }
}
