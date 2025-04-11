use x86_64::PhysAddr;

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
    pub fn parse_from_limine(&mut self, memory_map: &limine::response::MemoryMapResponse) {
        let entries = memory_map.entries();
        let size = entries.len().min(Self::SIZE as usize);
        self.size = size as u64;
        for (i, entry) in entries.enumerate() {
            if i >= size {
                break;
            }
            self.entries[i] = MemoryMapEntry::from(&entry);
        }
    }
}
