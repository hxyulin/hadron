use core::ops::{Index, IndexMut};

use limine::memory_map::{MemoryMapEntryType, MemoryMapIter};
use x86_64::{
    PhysAddr, VirtAddr,
    structures::paging::{PageSize, Size4KiB},
};

use crate::boot::arch::memory_map::{BootstrapMemoryMap, MemoryMapEntry, MemoryRegionType};

impl From<limine::memory_map::MemoryMapEntryType> for MemoryRegionType {
    fn from(entry_type: limine::memory_map::MemoryMapEntryType) -> Self {
        match entry_type {
            limine::memory_map::MemoryMapEntryType::Usable => Self::Usable,
            limine::memory_map::MemoryMapEntryType::Reserved => Self::Reserved,
            limine::memory_map::MemoryMapEntryType::AcpiReclaimable => Self::AcpiReclaimable,
            limine::memory_map::MemoryMapEntryType::AcpiNvs => Self::AcpiNvs,
            limine::memory_map::MemoryMapEntryType::BadMemory => Self::BadMemory,
            limine::memory_map::MemoryMapEntryType::BootloaderReclaimable => Self::BootloaderReclaimable,
            limine::memory_map::MemoryMapEntryType::KernelAndModules => Self::KernelAndModules,
            limine::memory_map::MemoryMapEntryType::Framebuffer => Self::Framebuffer,
        }
    }
}

impl From<&limine::memory_map::MemoryMapEntry> for MemoryMapEntry {
    fn from(entry: &limine::memory_map::MemoryMapEntry) -> Self {
        Self {
            base: PhysAddr::new(entry.base),
            length: entry.length,
            memory_type: entry.ty.into(),
        }
    }
}

impl BootstrapMemoryMap {
    pub fn parse_from_limine(&mut self, memory_map: &limine::response::MemoryMapResponse, hhdm_offset: u64) {
        self.init(memory_map.entries(), hhdm_offset);
    }

    /// Create a new memory map from a list of entries.
    ///
    /// This function does several things:
    /// 1. It finds a HHDM mapped region, which is long enough to hold the entire memory map.
    /// 2. It creates a frame based allocator with that frame.
    /// 3. It creates a vector of the memory map entries using the allocator
    /// 4. It marks that frame as used int he memory map.
    pub fn init(&mut self, limine_entries: MemoryMapIter, hhdm_offset: u64) {
        /// The number of entries we need to reserve for the memory map, for deallocation
        /// We copletely control this in the kernel, so this can be a constant
        const RESERVED_ENTRIES: usize = 8;
        let required_size = (size_of::<MemoryMapEntry>() * (limine_entries.len() + RESERVED_ENTRIES)) as u64;
        // Now we find a hhdm region (phys addr <= 4 GiB) that is long enough to hold the memory map
        const HHDM_END: u64 = 0x100000000;
        let region = limine_entries
            .clone()
            .find(|e| e.ty == MemoryMapEntryType::Usable && e.base <= HHDM_END && e.length >= required_size)
            .expect("memory map: requires a memory region that is long enough to hold the memory map");
        self.entries
            .allocator()
            .init(VirtAddr::new(region.base + hhdm_offset), region.length as usize);
        self.entries.reserve(limine_entries.len());

        for entry in limine_entries {
            let mut entry = MemoryMapEntry::from(entry);
            if entry.base.as_u64() == region.base {
                // Align the length to a page size, because everything else in the kernel assumes that
                // the memory map regions are page aligned
                let length = (region.length + Size4KiB::SIZE - 1) & !(Size4KiB::SIZE - 1);
                entry.base += length;
                entry.length -= length;
                if entry.length == 0 {
                    continue;
                }
            }
            self.entries.push(entry);
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
