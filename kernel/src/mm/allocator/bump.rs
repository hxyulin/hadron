use core::ptr::NonNull;

use crate::{arch::VirtAddr, mm::allocator::MutAllocator};

#[derive(Debug)]
pub struct BumpAllocator {
    base: VirtAddr,
    len: usize,
    index: usize,
}

impl BumpAllocator {
    pub const fn empty() -> Self {
        Self {
            base: VirtAddr::NULL,
            len: 0,
            index: 0,
        }
    }

    /// # Safety
    /// The caller must make sure the entire range (base..base + len) is readable and writable
    pub const unsafe fn new(base: VirtAddr, len: usize) -> Self {
        Self { base, len, index: 0 }
    }

    pub const fn init(&mut self, base: VirtAddr, len: usize) {
        self.base = base;
        self.len = len;
    }

    pub const fn mapped_range(&self) -> (VirtAddr, usize) {
        (self.base, self.len)
    }
}

unsafe impl MutAllocator for BumpAllocator {
    fn allocate(&mut self, layout: core::alloc::Layout) -> Result<NonNull<[u8]>, alloc::alloc::AllocError> {
        let index = self.index;
        self.index += layout.size();
        assert!(self.index <= self.len, "bump allocator is full!");
        // SAFETY: We checked that the index is valid, and the pointer should be valid
        let ptr = unsafe { core::slice::from_raw_parts_mut((self.base + index).as_mut_ptr(), layout.size()) };

        Ok(unsafe { NonNull::new_unchecked(ptr) })
    }

    unsafe fn deallocate(&mut self, _ptr: NonNull<u8>, _layout: core::alloc::Layout) {
        // For a bump allocator it is just a noop
    }
}
