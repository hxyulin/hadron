use core::{alloc::GlobalAlloc, ptr::NonNull};

use alloc::vec::Vec;
use spin::{Mutex, RwLock};
use x86_64::{
    VirtAddr,
    structures::paging::{PageSize, PageTableFlags, Size4KiB},
};

use crate::base::mem::mappings;

/// The kernel allocator.
/// This is a simple allocator that allocates memory from the kernel's memory map.
pub struct KernelAllocator {
    generic: Mutex<GenericAllocator>,
    zones: RwLock<Vec<ZoneAllocator>>,
}

pub struct GenericAllocator {
    // TODO: Create a custom allocator, and speicalize realloc.
    alloc: linked_list_allocator::Heap,
}

impl GenericAllocator {
    const EXPANSION_FACTOR: usize = 2;

    unsafe fn alloc(&mut self, layout: core::alloc::Layout) -> *mut u8 {
        match self.alloc.allocate_first_fit(layout) {
            Ok(allocation) => allocation.as_ptr(),
            Err(_) => {
                unsafe { self.grow() };
                self.alloc
                    .allocate_first_fit(layout)
                    .expect("Failed to allocate memory")
                    .as_ptr()
            }
        }
    }

    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: core::alloc::Layout) {
        debug_assert_ne!(ptr as usize, 0, "Tried to deallocate null pointer");
        unsafe { self.alloc.deallocate(NonNull::new_unchecked(ptr), layout) };
    }

    /// Grows the heap, and returns the new size of the heap.
    unsafe fn grow(&mut self) -> usize {
        let new_size = self.alloc.size() * Self::EXPANSION_FACTOR;
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
        new_size
    }
}

/// A zone allocator.
/// This is a simple allocator that allocates memory of a fixed size
pub struct ZoneAllocator {
    start: usize,
    length: usize,
    alloc_size: usize,
    bitmap: Mutex<Vec<u64>>,
}

impl ZoneAllocator {
    /// Allocate a new block in the zone.
    /// No arguments are needed because the zone allocator is a fixed size, and fixed alignment.
    unsafe fn alloc(&mut self) -> *mut u8 {
        todo!()
    }

    /// Deallocate a block in the zone.
    unsafe fn dealloc(&mut self, ptr: *mut u8) {
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
