//! Types for representing the memory map.

use core::ptr::NonNull;

/// A memory map entry.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MemoryMapEntry {
    /// The base physical address of the memory region.
    pub base: u64,
    /// The length of the memory region.
    pub length: u64,
    /// The type of the memory region.
    /// See [`MemoryMapEntryType`] for more information.
    pub ty: MemoryMapEntryType,
}

/// The type of a memory map entry.
#[repr(u64)]
#[derive(Debug, Clone, Copy)]
pub enum MemoryMapEntryType {
    /// A usable memory region.
    Usable = 0,
    /// A reserved memory region.
    Reserved = 1,
    /// An ACPI reclaimable memory region.
    /// This can be reclaimed after reading the ACPI tables.
    AcpiReclaimable = 2,
    /// An ACPI NVS memory region.
    AcpiNvs = 3,
    /// A bad memory region.
    BadMemory = 4,
    /// A memory region that can be reclaimed after the bootloader info is no longer needed.
    BootloaderReclaimable = 5,
    /// A memory region that is used by the kernel and modules.
    KernelAndModules = 6,
    /// A memory region that is used by the framebuffer.
    Framebuffer = 7,
}

/// An iterator over the memory map.
pub struct MemoryMapIter<'a> {
    memory_map: &'a [NonNull<MemoryMapEntry>],
    index: usize,
}

impl<'a> MemoryMapIter<'a> {
    /// Creates a new `MemoryMapIter` from a slice of memory map pointers.
    pub(crate) fn new(memory_map: &'a [NonNull<MemoryMapEntry>]) -> Self {
        Self { memory_map, index: 0 }
    }

    #[cfg(feature = "internal-api")]
    pub fn internal_new(memory_map: &'a [NonNull<MemoryMapEntry>]) -> Self {
        Self { memory_map, index: 0 }
    }

    pub fn len(&self) -> usize {
        self.memory_map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.memory_map.is_empty()
    }
}

impl<'a> Iterator for MemoryMapIter<'a> {
    type Item = MemoryMapEntry;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.memory_map.len() {
            return None;
        }
        self.index += 1;
        // SAFETY: The memory map pointer is valid because it is a pointer to a memory map entry.
        Some(unsafe { self.memory_map[self.index - 1].read_volatile() })
    }
}
