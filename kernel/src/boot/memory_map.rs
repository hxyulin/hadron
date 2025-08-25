use core::ops::{Index, IndexMut};

use crate::{
    arch::{PhysAddr, VirtAddr},
    boot::frame_allocator::BootstrapFrameAllocator,
    kprintln,
    mm::{
        allocator::{Locked, Shared, bump::BumpAllocator},
        mappings,
        memory_map::{MemoryMap, MemoryRegion, MemoryRegionTag},
        page_table::{KernelPageTable, Mapper, PageTableFlags},
        paging::{FrameAllocator, Page, PageSize, Size2MiB, Size4KiB},
    },
};
use alloc::vec::Vec;

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
    pub(crate) length: usize,
    // We can make the memory type 8bit, and have 56 more for future use
    pub(crate) memory_type: MemoryRegionType,
}

pub struct UsableRegion {
    pub base: PhysAddr,
    pub f_pad_4kib: usize,
    pub f_pad_2mib: usize,
    pub mid_1gib: usize,
    pub b_pad_2mib: usize,
    pub b_pad_4kib: usize,
}

impl Default for UsableRegion {
    fn default() -> Self {
        Self {
            base: PhysAddr::NULL,
            f_pad_4kib: 0,
            f_pad_2mib: 0,
            mid_1gib: 0,
            b_pad_2mib: 0,
            b_pad_4kib: 0,
        }
    }
}

impl UsableRegion {
    pub fn from_region(region: &limine::memory_map::MemoryMapEntry) -> Option<Self> {
        use limine::memory_map::MemoryMapEntryType;
        if region.ty != MemoryMapEntryType::Usable {
            return None;
        }

        let start = PhysAddr::new(region.base as usize);
        let end = start + (region.length as usize);
        debug_assert!(start.is_aligned(Size4KiB::SIZE), "memory regions are not aligned!");
        let aligned_start = start.align_up(Size2MiB::SIZE);
        if end < (aligned_start + Size2MiB::SIZE) {
            // We can't fit 2MiB, so it will have to fit 4KiB pages
            return Some(Self {
                f_pad_4kib: (end - start) / Size4KiB::SIZE,
                ..Default::default()
            });
        }
        let f_pad_4kib = (aligned_start - start) / Size4KiB::SIZE;
        // TODO: 1GiB pages are not yet supported
        let aligned_end = end.align_down(Size2MiB::SIZE);
        let f_pad_2mib = (aligned_end - aligned_start) / Size2MiB::SIZE;
        let b_pad_4kib = (end - aligned_end) / Size4KiB::SIZE;
        return Some(Self {
            f_pad_4kib,
            f_pad_2mib,
            b_pad_4kib,
            ..Default::default()
        });
    }

    pub fn pages_needed(&self) -> usize {
        let total = self.f_pad_4kib + self.f_pad_2mib + self.mid_1gib + self.b_pad_2mib + self.b_pad_4kib;
        3 + total + total.div_ceil(512)
    }
}

impl MemoryMapEntry {
    pub const fn new(base: PhysAddr, length: usize, memory_type: MemoryRegionType) -> Self {
        Self {
            base,
            length,
            memory_type,
        }
    }

    pub fn base(&self) -> PhysAddr {
        self.base
    }

    pub fn length(&self) -> usize {
        self.length
    }

    pub fn end(&self) -> PhysAddr {
        self.base + self.length
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
    pub entries: Vec<MemoryMapEntry, Locked<BumpAllocator>>,
}

impl BootstrapMemoryMap {
    pub const fn empty() -> Self {
        Self {
            entries: Vec::new_in(Locked::new(BumpAllocator::empty())),
        }
    }

    pub fn total_size(&self) -> usize {
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
    pub fn mapped_range(&self) -> (VirtAddr, usize) {
        self.entries.allocator().lock().mapped_range()
    }
}

pub trait MainMemoryMap {
    fn from_bootstrap(memory_map: &mut BootstrapMemoryMap, page_table: &mut KernelPageTable) -> Self;
    fn push_entry(&mut self, entry: MemoryMapEntry);
}

impl MainMemoryMap for MemoryMap {
    /// Parses the memory map from the bootstrap info
    ///
    /// This will allocate at most [`Self::MAX_BOOTSTRAP_PAGES`] pages for the kernel heap,
    /// storing the rest of the usable memory in the `special` field
    fn from_bootstrap(memory_map: &mut BootstrapMemoryMap, page_table: &mut KernelPageTable) -> Self {
        /// The number of entries we need to reserve for the memory map, for deallocation of the
        /// bootstrap structures
        const RESERVED_ENTRIES: usize = 8;
        // We need to calculate how much memory we need for the entire memory map
        let mut required_size = size_of::<MemoryRegion>() * RESERVED_ENTRIES;
        for region in memory_map.iter() {
            required_size += size_of::<MemoryRegion>();
            let bitmap_size = region.length.div_ceil(32768);
            // We align it to 64 bytes to be conservative
            required_size += (bitmap_size + 63) & !63;
        }

        let mut frame_allocator = BootstrapFrameAllocator::new(memory_map);
        let required_frames = required_size.div_ceil(Size4KiB::SIZE);
        for i in 0..required_frames {
            let addr = mappings::MEMORY_MAPPINGS + i * Size4KiB::SIZE;
            unsafe {
                kprintln!(Debug, "mapping to: {:#x}", addr);
                page_table.map_with_allocator(
                    Page::from_start_address(addr),
                    frame_allocator.allocate_frame().unwrap(),
                    PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_EXECUTE,
                    &mut frame_allocator,
                )
            };
        }

        let alloc = unsafe {
            Shared::new(Locked::new(BumpAllocator::new(
                mappings::MEMORY_MAPPINGS,
                required_frames * Size4KiB::SIZE,
            )))
        };
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

    fn push_entry(&mut self, entry: MemoryMapEntry) {
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

impl Index<usize> for BootstrapMemoryMap {
    type Output = MemoryMapEntry;

    fn index(&self, index: usize) -> &Self::Output {
        &self.entries[index]
    }
}

impl IndexMut<usize> for BootstrapMemoryMap {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.entries[index]
    }
}
