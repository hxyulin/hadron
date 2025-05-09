use super::allocator::FrameBasedAllocator;
use crate::base::mem::sync::Arc;
use alloc::vec::Vec;
use x86_64::{
    PhysAddr,
    structures::paging::{PageSize, Size4KiB},
};

#[derive(Debug)]
pub struct MemoryMap {
    pub alloc: Arc<FrameBasedAllocator>,
    pub entries: Vec<MemoryRegion, Arc<FrameBasedAllocator>>,
    pub special: Vec<SpecialMemoryRegion>,
}

#[derive(Debug, Clone)]
pub struct MemoryRegion {
    pub(super) base: PhysAddr,
    pub(super) bitmap: Bitmap<Arc<FrameBasedAllocator>>,
}

#[derive(Clone)]
pub struct Bitmap<A: alloc::alloc::Allocator = alloc::alloc::Global>(Vec<u64, A>, usize);

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
