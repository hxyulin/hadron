use core::ptr::NonNull;

// FIXME: We should not use dependencies as a core design goal for the kernel
use linked_list_allocator::Heap;

use crate::mm::allocator::MutGlobalAlloc;

pub struct LinkedListAllocator {
    inner: Heap,
}

impl LinkedListAllocator {
    pub const fn empty() -> Self {
        Self { inner: Heap::empty() }
    }

    pub unsafe fn init(&mut self, heap_bottom: *mut u8, heap_size: usize) {
        unsafe {
            self.inner.init(heap_bottom, heap_size);
        }
    }
}

unsafe impl MutGlobalAlloc for LinkedListAllocator {
    unsafe fn alloc(&mut self, layout: core::alloc::Layout) -> *mut u8 {
        match self.inner.allocate_first_fit(layout) {
            Ok(ptr) => ptr.as_ptr(),
            Err(_) => core::ptr::null_mut(),
        }
    }

    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: core::alloc::Layout) {
        unsafe { self.inner.deallocate(NonNull::new(ptr).unwrap(), layout) };
    }
}
