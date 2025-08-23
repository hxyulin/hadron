use core::{alloc::GlobalAlloc, ops::DerefMut, ptr::NonNull};
use spin::{Mutex, MutexGuard};

pub mod bump;
pub mod no_alloc;

pub struct Locked<T> {
    alloc: Mutex<T>,
}

impl<T> Locked<T> {
    pub const fn new(alloc: T) -> Self {
        Self {
            alloc: Mutex::new(alloc),
        }
    }

    pub fn call<F, O>(&self, f: F) -> O
    where
        F: FnOnce(&mut T) -> O,
    {
        let mut alloc = self.alloc.lock();
        f(&mut alloc)
    }

    pub fn lock(&self) -> MutexGuard<'_, T> {
        self.alloc.lock()
    }
}

pub unsafe trait MutAllocator {
    fn allocate(&mut self, layout: core::alloc::Layout) -> Result<NonNull<[u8]>, alloc::alloc::AllocError>;
    unsafe fn deallocate(&mut self, ptr: NonNull<u8>, layout: core::alloc::Layout);
}

unsafe impl<T> alloc::alloc::Allocator for Locked<T>
where
    T: MutAllocator,
{
    fn allocate(&self, layout: core::alloc::Layout) -> Result<NonNull<[u8]>, alloc::alloc::AllocError> {
        MutAllocator::allocate(self.alloc.lock().deref_mut(), layout)
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: core::alloc::Layout) {
        unsafe { MutAllocator::deallocate(self.alloc.lock().deref_mut(), ptr, layout) }
    }
}

#[global_allocator]
static ALLOCATOR: KernelAllocator = KernelAllocator::new();

pub struct KernelAllocator {}

impl KernelAllocator {
    pub const fn new() -> Self {
        Self {}
    }
}

unsafe impl GlobalAlloc for KernelAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        todo!()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        todo!()
    }
}
