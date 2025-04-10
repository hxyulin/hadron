use x86_64::PhysAddr;

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
    const fn default() -> Self {
        Self {
            base: PhysAddr::new(0),
            length: 0,
            memory_type: MemoryRegionType::Usable,
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

#[derive(Clone)]
pub struct MemoryMap {
    // TODO: Instead of storing fixed size, we can store usable entries and just store a reference to the actual memory map for the bootloader until we have the heap
    pub(crate) size: u64,
    pub(crate) entries: [MemoryMapEntry; Self::SIZE],
}

impl core::fmt::Debug for MemoryMap {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut debug_list = f.debug_list();
        for (idx, entry) in self.iter().enumerate() {
            if idx > self.size as usize {
                break;
            }
            debug_list.entry(entry);
        }
        debug_list.finish()
    }
}

impl MemoryMap {
    pub const fn default() -> Self {
        Self {
            size: 0,
            entries: [MemoryMapEntry::default(); Self::SIZE],
        }
    }

    pub fn reclaim_bootloader_memory(&mut self) {
        for region in self.iter_mut() {
            if region.ty() == MemoryRegionType::BootloaderReclaimable {
                region.memory_type = MemoryRegionType::Usable;
            }
        }
    }
}

impl MemoryMap {
    pub const SIZE: usize = 128;

    pub fn iter(&self) -> MemoryMapIter {
        MemoryMapIter {
            end: unsafe { self.entries.as_ptr().add(Self::SIZE) },
            current: self.entries.as_ptr(),
            phantom: core::marker::PhantomData,
        }
    }

    pub fn iter_mut(&mut self) -> MemoryMapIterMut {
        MemoryMapIterMut {
            end: unsafe { self.entries.as_mut_ptr().add(Self::SIZE) },
            current: self.entries.as_mut_ptr(),
            phantom: core::marker::PhantomData,
        }
    }
}

pub struct MemoryMapIter<'a> {
    end: *const MemoryMapEntry,
    current: *const MemoryMapEntry,
    phantom: core::marker::PhantomData<&'a MemoryMap>,
}

pub struct MemoryMapIterMut<'a> {
    end: *mut MemoryMapEntry,
    current: *mut MemoryMapEntry,
    phantom: core::marker::PhantomData<&'a mut MemoryMap>,
}

impl<'a> Iterator for MemoryMapIter<'a> {
    type Item = &'a MemoryMapEntry;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.end {
            return None;
        }
        let entry = unsafe { self.current.as_ref().unwrap() };
        self.current = unsafe { self.current.add(1) };
        Some(entry)
    }
}

impl<'a> Iterator for MemoryMapIterMut<'a> {
    type Item = &'a mut MemoryMapEntry;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.end {
            return None;
        }
        let entry = unsafe { self.current.as_mut().unwrap() };
        self.current = unsafe { self.current.add(1) };
        Some(entry)
    }
}
