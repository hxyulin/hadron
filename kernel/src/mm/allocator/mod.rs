use alloc::{alloc::Allocator, sync::Arc};
use core::{alloc::GlobalAlloc, fmt::Debug, ops::DerefMut, ptr::NonNull};
use spin::{Mutex, MutexGuard};

use crate::mm::allocator::linked_list::LinkedListAllocator;

pub mod bump;
pub mod linked_list;
pub mod no_alloc;

pub type SharedLock<T> = Shared<Locked<T>>;

pub struct Locked<T> {
    alloc: Mutex<T>,
}

impl<T: Debug> Debug for Locked<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(&self.alloc, f)
    }
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

pub struct Shared<T: Allocator> {
    alloc: Arc<T>,
}

impl<T: Allocator> Clone for Shared<T> {
    fn clone(&self) -> Self {
        Self {
            alloc: self.alloc.clone(),
        }
    }
}

impl<T: Debug + Allocator> Debug for Shared<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(&self.alloc, f)
    }
}

impl<T> Shared<T>
where
    T: Allocator,
{
    pub fn new(alloc: T) -> Self {
        Self { alloc: Arc::new(alloc) }
    }
}

unsafe impl<T> Allocator for Shared<T>
where
    T: Allocator,
{
    fn allocate(&self, layout: core::alloc::Layout) -> Result<NonNull<[u8]>, alloc::alloc::AllocError> {
        self.alloc.allocate(layout)
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: core::alloc::Layout) {
        unsafe { self.alloc.deallocate(ptr, layout) };
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

pub unsafe trait MutGlobalAlloc {
    unsafe fn alloc(&mut self, layout: core::alloc::Layout) -> *mut u8;
    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: core::alloc::Layout);
}

unsafe impl<T> alloc::alloc::GlobalAlloc for Locked<T>
where
    T: MutGlobalAlloc,
{
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        unsafe { MutGlobalAlloc::alloc(self.alloc.lock().deref_mut(), layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        unsafe { MutGlobalAlloc::dealloc(self.alloc.lock().deref_mut(), ptr, layout) }
    }
}

#[global_allocator]
pub static ALLOCATOR: KernelAllocator = KernelAllocator::new();

pub struct KernelAllocator {
    generic: Locked<LinkedListAllocator>,
}

impl KernelAllocator {
    pub const fn new() -> Self {
        Self {
            generic: Locked::new(LinkedListAllocator::empty()),
        }
    }

    pub unsafe fn init(&self, addr: *mut u8, size: usize) {
        unsafe { self.generic.lock().init(addr, size) };
    }
}

unsafe impl GlobalAlloc for KernelAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        unsafe { GlobalAlloc::alloc(&self.generic, layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        unsafe { GlobalAlloc::dealloc(&self.generic, ptr, layout) }
    }
}
