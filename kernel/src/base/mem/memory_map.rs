use crate::{
    base::info::kernel_info,
    boot::arch::memory_map::{BootstrapMemoryMap, MemoryRegionType},
};
use alloc::vec::Vec;
use x86_64::{
    PhysAddr,
    structures::paging::{PageSize, Size4KiB},
};

#[derive(Debug, Clone)]
pub struct MemoryMap {
    pub(super) entries: Vec<MemoryRegion>,
    pub(super) special: Vec<SpecialMemoryRegion>,
}

impl MemoryMap {}

#[derive(Debug, Clone)]
pub struct MemoryRegion {
    pub(super) base: PhysAddr,
    pub(super) bitmap: Bitmap,
}

impl MemoryRegion {
    pub fn from_base_and_length(base: PhysAddr, length: u64) -> Self {
        let pages = length / Size4KiB::SIZE;
        let bitmap = Bitmap::new(pages as usize);
        Self { base, bitmap }
    }

    pub fn pages(&self) -> u64 {
        self.bitmap.size() as u64
    }
}

impl MemoryRegion {
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

    unsafe fn resize(&mut self, new_size_pages: usize) {
        unsafe { self.bitmap.resize(new_size_pages) };
    }
}

#[derive(Clone)]
pub struct Bitmap(Vec<u64>, usize);

impl Bitmap {
    pub fn new(size: usize) -> Self {
        let len = size.div_ceil(64);
        Self(alloc::vec![0; len], size)
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

impl core::fmt::Debug for Bitmap {
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

pub struct MemoryMapBootstrapInfo {
    /// Index of all the memory regions that were boostrapped
    used: Vec<usize>,
    /// If the last region was partially allocated, this will contain the size of the last region
    /// that was used
    size: Option<usize>,
    total_pages: usize,
}

impl MemoryMapBootstrapInfo {
    fn new() -> Self {
        Self {
            used: Vec::new(),
            size: None,
            total_pages: 0,
        }
    }
}

impl MemoryMap {
    /// The maximum number of pages that can be allocated for bootstrapping
    /// This is a safety measure to prevent the kernel from running out of memory
    /// during the bootstrapping process, before the kernel heap can grow
    const MAX_BOOTSTRAP_PAGES: usize = 4096;

    /// The maximum number of special regions that can be allocated
    /// This is a safety measure to prevent the kernel from running out of memory
    /// during the bootstrapping process, before the kernel heap can grow
    const MAX_SPECIAL_REGIONS: usize = 64;

    /// Parses the memory map from the bootstrap info
    ///
    /// This will allocate at most [`Self::MAX_BOOTSTRAP_PAGES`] pages for the kernel heap,
    /// storing the rest of the usable memory in the `special` field
    pub fn from_bootstrap(memory_map: &BootstrapMemoryMap) -> Self {
        let mut entries = Vec::new();
        let mut special = Vec::new();
        let mut used_pages = 0;

        for entry in memory_map.iter() {
            assert!(special.len() < Self::MAX_SPECIAL_REGIONS, "Too many special regions");
            if entry.length == 0 {
                continue;
            }
            if entry.ty() == MemoryRegionType::Usable {
                let pages = (entry.length / Size4KiB::SIZE) as usize;
                if used_pages >= Self::MAX_BOOTSTRAP_PAGES {
                    special.push(SpecialMemoryRegion {
                        base: entry.base,
                        length: entry.length,
                        tag: MemoryRegionTag::Unallocated,
                    });
                    continue;
                }
                if used_pages + pages > Self::MAX_BOOTSTRAP_PAGES {
                    let used_size = (Self::MAX_BOOTSTRAP_PAGES - used_pages) as u64 * Size4KiB::SIZE;
                    used_pages = Self::MAX_BOOTSTRAP_PAGES;
                    entries.push(MemoryRegion::from_base_and_length(entry.base(), used_size));
                    special.push(SpecialMemoryRegion {
                        base: entry.base + used_size,
                        length: entry.length - used_size,
                        tag: MemoryRegionTag::Unallocated,
                    });
                    continue;
                }

                entries.push(MemoryRegion::from_base_and_length(entry.base(), entry.length()));
            } else if let Some(tag) = MemoryRegionTag::from_type(entry.ty()) {
                special.push(SpecialMemoryRegion {
                    base: entry.base(),
                    length: entry.length(),
                    tag,
                });
            }
        }
        entries.shrink_to_fit();
        special.shrink_to_fit();

        Self { entries, special }
    }
}

fn allocate(to_allocate: u64) {
    let frame_allocator = &kernel_info().frame_allocator;
    let mut idx = 0;
    let mut allocated = 0;
    while allocated < to_allocate {
        let entry = {
            let frame_allocator = frame_allocator.lock();
            let memory_map = frame_allocator.memory_map();
            if idx >= memory_map.special.len() {
                // We have allocated all the memory
                break;
            }
            if memory_map.special[idx].tag != MemoryRegionTag::Unallocated {
                idx += 1;
                continue;
            }
            memory_map.special[idx].clone()
        };
        let size = entry.length;
        let to_allocate = to_allocate - allocated;
        if size <= to_allocate {
            // We just pop the region
            let region = MemoryRegion::from_base_and_length(entry.base, entry.length);
            let mut frame_allocator = frame_allocator.lock();
            let memory_map = unsafe { frame_allocator.memory_map_mut() };
            memory_map.special.remove(idx);
            memory_map.entries.push(region);
            allocated += size;
        } else {
            let region = MemoryRegion::from_base_and_length(entry.base, to_allocate);
            let mut frame_allocator = frame_allocator.lock();
            let memory_map = unsafe { frame_allocator.memory_map_mut() };
            memory_map.entries.push(region);
            memory_map.special[idx].length -= to_allocate;
            memory_map.special[idx].base += to_allocate;
            break;
        }
    }
}

/// Finishes the bootstrapping process by allocating the rest of the memory
/// This is a complicated process, since we need enough memory to allocate the memory map,
/// which may require us to allocate more memory. It is a recursive process, allocating chunks
/// until we can allocate the full memory map
pub fn finish_bootstrap() {
    let mut required_sizes = Vec::new();
    let mut required_size: u64 = {
        let frame_allocator = kernel_info().frame_allocator.lock();
        frame_allocator
            .memory_map()
            .special
            .iter()
            .filter_map(|r| {
                if r.tag == MemoryRegionTag::Unallocated {
                    Some(r.length)
                } else {
                    None
                }
            })
            .sum()
    };
    // We need to be able to allocate the memory map, so we assume that we can use at most 50% of the memory
    let current_size = crate::ALLOCATOR.generic_size() as u64 / 2;
    log::debug!("Current size: {:#X}", current_size);
    while required_size > current_size {
        required_sizes.push(required_size);
        // FIXME: This is very naive implementation
        // The size required is 1 byte for every 8 pages = factor of 32768, but we grow by a factor of 2, so we need to divide by 2
        // IMPORTANT/FIXME: We don't account for the metadata here, which may be a problem for some sizes
        required_size = required_size.div_ceil(32768 / 2) + 4096;
    }

    for size in required_sizes.into_iter().rev() {
        log::debug!("Allocating {:#X} bytes", size);
        allocate(size);
    }
}
