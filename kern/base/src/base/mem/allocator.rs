use core::{alloc::GlobalAlloc, ptr::NonNull};

use alloc::{alloc::Allocator, collections::btree_map::BTreeMap, vec::Vec};
use linked_list_allocator::LockedHeap;
use spin::{Mutex, RwLock};
use x86_64::{
    VirtAddr,
    structures::paging::{PageSize, PageTableFlags, Size4KiB},
};

use crate::base::mem::mappings;

use super::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ZoneId(usize);

/// The kernel allocator.
/// This is a simple allocator that allocates memory from the kernel's memory map.
pub struct KernelAllocator {
    generic: Mutex<GenericAllocator>,
    /// The zone allocators
    ///
    /// This is a list of zone allocators.
    /// This list is designed to stay relatively constant in size,
    /// but allowing for the possibility of adding more zone allocators.
    /// Removal of zone allocators is not supported.
    zones: RwLock<Vec<ZoneAllocator>>,
    zone_indices: RwLock<BTreeMap<&'static str, ZoneId>>,
}

impl KernelAllocator {
    pub const fn empty() -> Self {
        Self {
            generic: Mutex::new(GenericAllocator::empty()),
            zones: RwLock::new(Vec::new()),
            zone_indices: RwLock::new(BTreeMap::new()),
        }
    }

    pub fn generic_size(&self) -> usize {
        self.generic.lock().size()
    }

    /// Initializes the generic allocator with the given heap.
    ///
    /// # Safety
    /// This function is unsafe because it can cause UB if the heap is not valid, or called more than once.
    pub unsafe fn init_generic(&self, heap_start: *mut u8, heap_size: usize) {
        let mut generic = self.generic.lock();
        unsafe { generic.init(heap_start, heap_size) };
    }

    /// Grows the generic allocator by the given size.
    pub fn grow_generic(&self, grow_size: usize) {
        let mut generic = self.generic.lock();
        generic.grow_by_size(grow_size);
    }

    /// Creates a zone allocator.
    ///
    /// # Arguments
    /// - `ident`: The identifier of the zone allocator.
    /// - `initial_size`: The initial size of the zone allocator.
    /// - `alloc_size`: The allocation size of the zone allocator.
    pub fn create_zone(&self, _ident: &'static str, _initial_size: usize, _alloc_size: usize) -> ZoneId {
        todo!()
    }

    pub fn get_zone_id(&self, ident: &str) -> Option<ZoneId> {
        self.zone_indices.read().get(ident).copied()
    }

    pub fn get_zone_info(&self, id: ZoneId) -> Option<ZoneInfo> {
        self.zones.read().get(id.0).map(ZoneAllocator::info)
    }
}

pub struct GenericAllocator {
    // TODO: Create a custom allocator, and speicalize realloc.
    alloc: linked_list_allocator::Heap,
}

impl GenericAllocator {
    const EXPANSION_FACTOR: usize = 2;

    pub const fn empty() -> Self {
        Self {
            alloc: linked_list_allocator::Heap::empty(),
        }
    }

    pub fn size(&self) -> usize {
        self.alloc.size()
    }

    /// Initializes the allocator with the given heap.
    ///
    /// # Safety
    /// This function is unsafe because it can cause UB if the heap is not valid, or aclled more than once.
    pub unsafe fn init(&mut self, heap_start: *mut u8, heap_size: usize) {
        unsafe { self.alloc.init(heap_start, heap_size) };
    }

    unsafe fn alloc(&mut self, layout: core::alloc::Layout) -> *mut u8 {
        loop {
            match self.alloc.allocate_first_fit(layout) {
                Ok(allocation) => return allocation.as_ptr(),
                Err(_) => self.grow(),
            };
        }
    }

    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: core::alloc::Layout) {
        debug_assert_ne!(ptr as usize, 0, "Tried to deallocate null pointer");
        unsafe { self.alloc.deallocate(NonNull::new_unchecked(ptr), layout) };
    }

    /// Grows the heap, and returns the new size of the heap.
    fn grow(&mut self) -> usize {
        let new_size = self.alloc.size() * Self::EXPANSION_FACTOR - self.alloc.size();
        log::trace!("KEREL: Heap grown to {:#X}b", new_size);
        self.grow_by_size(new_size);
        new_size
    }

    /// Grows the heap by the given size.
    fn grow_by_size(&mut self, grow_size: usize) {
        // FIXME: This should grow as much as possible, but it just panics if we run out of memory.
        let new_size = self.alloc.size() + grow_size;
        assert!(new_size <= mappings::KERNEL_HEAP_SIZE as usize, "Heap is full");
        let extra_pages = (new_size - self.alloc.size()).div_ceil(Size4KiB::SIZE as usize);
        let heap_end = self.alloc.top() as u64;
        for i in 0..extra_pages as u64 {
            let offset = heap_end + i * Size4KiB::SIZE;
            let frame = super::alloc_frame().expect("Failed to allocate frame");
            unsafe {
                super::map_page(
                    frame,
                    VirtAddr::new(offset),
                    PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_EXECUTE,
                )
            };
        }
        unsafe { self.alloc.extend(grow_size) };
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ZoneInfo {
    start: VirtAddr,
    length: usize,
    alloc_size: usize,
}

/// A zone allocator.
/// This is a simple allocator that allocates memory of a fixed size
pub struct ZoneAllocator {
    start: VirtAddr,
    length: usize,
    alloc_size: usize,
    bitmap: Mutex<Vec<u64>>,
}

impl ZoneAllocator {
    pub fn info(&self) -> ZoneInfo {
        ZoneInfo {
            start: self.start,
            length: self.length,
            alloc_size: self.alloc_size,
        }
    }

    pub fn alloc_size(&self) -> usize {
        self.alloc_size
    }

    /// Allocate a new block in the zone.
    /// No arguments are needed because the zone allocator is a fixed size, and fixed alignment.
    unsafe fn alloc(&mut self) -> *mut u8 {
        todo!()
    }

    /// Deallocate a block in the zone.
    unsafe fn dealloc(&mut self, _ptr: *mut u8) {
        todo!()
    }
}

unsafe impl GlobalAlloc for KernelAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        unsafe { self.generic.lock().alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        unsafe { self.generic.lock().dealloc(ptr, layout) }
    }
}

/// Allocates a fixed size array using a zone allocator.
pub fn z_alloc(_id: ZoneId) -> *mut u8 {
    todo!()
}

/// Deallocates a fixed size array using a zone allocator.
pub fn z_dealloc(_id: ZoneId, _ptr: *mut u8) {
    todo!()
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
            Err(_) => Err(alloc::alloc::AllocError),
        }
    }

    unsafe fn deallocate(&self, ptr: core::ptr::NonNull<u8>, layout: core::alloc::Layout) {
        unsafe { self.heap.lock().deallocate(ptr.cast(), layout) }
    }
}
