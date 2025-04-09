use core::ptr::NonNull;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MemoryMapEntry {
    pub base: u64,
    pub length: u64,
    pub ty: MemoryMapEntryType,
}

#[repr(u64)]
#[derive(Debug, Clone, Copy)]
pub enum MemoryMapEntryType {
    Usable = 0,
    Reserved = 1,
    AcpiReclaimable = 2,
    AcpiNvs = 3,
    BadMemory = 4,
    BootloaderReclaimable = 5,
    KernelAndModule = 6,
    Framebuffer = 7,
}

pub struct MemoryMapIter<'a> {
    memory_map: &'a [NonNull<MemoryMapEntry>],
    index: usize,
}

impl<'a> MemoryMapIter<'a> {
    pub(crate) fn new(memory_map: &'a [NonNull<MemoryMapEntry>]) -> Self {
        Self { memory_map, index: 0 }
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
