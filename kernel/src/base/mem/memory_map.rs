use crate::boot::arch::memory_map::{MemoryMap as BootstrapMemoryMap, MemoryRegionType};
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
    pub(super) pages: u64,
}

impl MemoryRegion {
    pub fn from_base_and_length(base: PhysAddr, length: u64) -> Self {
        let pages = length / Size4KiB::SIZE;
        let bitmap = Bitmap::new(pages as usize);
        Self { base, bitmap, pages }
    }
}

impl MemoryRegion {
    pub(super) fn contains(&self, addr: PhysAddr) -> bool {
        addr >= self.base && addr < self.base + self.pages * Size4KiB::SIZE
    }

    pub(super) fn allocate(&mut self) -> Option<usize> {
        let idx = self.bitmap.find_free()?;
        self.bitmap.set(idx, true);
        Some(idx)
    }

    pub(super) fn deallocate(&mut self, idx: usize) {
        self.bitmap.set(idx, false);
    }
}

#[derive(Clone)]
pub struct Bitmap(Vec<u64>);

impl Bitmap {
    pub fn new(size: usize) -> Self {
        let len = size.div_ceil(64);
        Self(alloc::vec![0; len])
    }

    pub fn set(&mut self, idx: usize, value: bool) {
        let idx = idx / 64;
        let bit = idx % 64;
        let byte = idx / 8;
        if value {
            self.0[byte] |= 1 << bit;
        } else {
            self.0[byte] &= !(1 << bit);
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
                if (byte & (1 << bit)) == 0 {
                    return Some(idx * 64 + bit);
                }
            }
        }
        None
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

impl MemoryMap {
    pub fn from_bootstrap(memory_map: &BootstrapMemoryMap) -> Self {
        let mut entries = Vec::new();
        let mut special = Vec::new();
        for entry in memory_map.iter() {
            if entry.length == 0 {
                continue;
            }

            if entry.ty() == MemoryRegionType::Usable {
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
