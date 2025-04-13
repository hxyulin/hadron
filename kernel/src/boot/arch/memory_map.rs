use core::ptr::NonNull;

use alloc::{
    alloc::{AllocError, Allocator},
    vec::Vec,
};
use linked_list_allocator::LockedHeap;
use x86_64::{PhysAddr, VirtAddr};

use crate::base::mem::sync::Arc;

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
    const fn default() -> Self {
        Self {
            base: PhysAddr::new(0),
            length: 0,
            memory_type: MemoryRegionType::Usable,
        }
    }

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

pub struct FrameBasedAllocator {
    // TODO: Make this a bump allocator or something
    heap: LockedHeap,
}

impl core::fmt::Debug for FrameBasedAllocator {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("FrameBasedAllocator").finish()
    }
}

impl FrameBasedAllocator {
    pub const fn empty() -> Self {
        Self {
            heap: LockedHeap::empty(),
        }
    }

    pub unsafe fn new(base: VirtAddr, length: usize) -> Self {
        Self {
            heap: unsafe { LockedHeap::new(base.as_mut_ptr(), length) },
        }
    }

    pub fn init(&self, base: VirtAddr, length: usize) {
        unsafe { self.heap.lock().init(base.as_mut_ptr(), length) };
    }

    pub fn mapped_range(&self) -> (VirtAddr, u64) {
        let heap = self.heap.lock();
        (VirtAddr::new(heap.bottom() as u64), heap.size() as u64)
    }

    pub fn deinit(self, dealloc: &mut BasicFrameAllocator, hhdm_offset: u64) {
        let (base, length) = self.mapped_range();
        dealloc.deallocate_region(PhysAddr::new((base - hhdm_offset).as_u64()), length);
    }
}

unsafe impl Allocator for Arc<FrameBasedAllocator> {
    fn allocate(&self, layout: core::alloc::Layout) -> Result<core::ptr::NonNull<[u8]>, alloc::alloc::AllocError> {
        (**self).allocate(layout)
    }

    unsafe fn deallocate(&self, ptr: core::ptr::NonNull<u8>, layout: core::alloc::Layout) {
        unsafe { (**self).deallocate(ptr, layout) }
    }
}

unsafe impl Allocator for FrameBasedAllocator {
    fn allocate(&self, layout: core::alloc::Layout) -> Result<core::ptr::NonNull<[u8]>, alloc::alloc::AllocError> {
        match self.heap.lock().allocate_first_fit(layout) {
            Ok(ptr) => Ok(NonNull::slice_from_raw_parts(ptr, layout.size())),
            Err(_) => Err(AllocError),
        }
    }

    unsafe fn deallocate(&self, ptr: core::ptr::NonNull<u8>, layout: core::alloc::Layout) {
        unsafe { self.heap.lock().deallocate(ptr.cast(), layout) }
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
