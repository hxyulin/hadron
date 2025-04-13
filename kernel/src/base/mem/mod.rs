use core::{marker::Unsize, ops::{CoerceUnsized, DispatchFromDyn}};

use x86_64::{
    VirtAddr,
    structures::paging::{FrameAllocator, FrameDeallocator, Page, PageTableFlags, PhysFrame},
};

use super::info::kernel_info;

pub mod allocator;
pub mod frame_allocator;
pub mod mappings;
pub mod memory_map;
pub mod page_table;

/// Allocates a frame.
pub fn alloc_frame() -> Option<PhysFrame> {
    kernel_info().frame_allocator.lock().allocate_frame()
}

/// Frees a frame.
///
/// # Safety
/// This function is unsafe because the frame should not be in use when it is freed.
pub unsafe fn free_frame(frame: PhysFrame) {
    unsafe { kernel_info().frame_allocator.lock().deallocate_frame(frame) };
}

/// Maps a page to a frame.
///
/// # Safety
/// This function is unsafe because mapping the same physical frame to multiple virtual addresses can cause UB.
pub unsafe fn map_page(frame: PhysFrame, addr: VirtAddr, flags: PageTableFlags) {
    let page = Page::from_start_address(addr).expect("map_page should be called with aligned addresses");
    unsafe { kernel_info().page_table.lock().map(page, frame, flags) };
}

/// Unmaps a page.
///
/// # Safety
/// This function is unsafe because it can cause UB if the page is still in use.
pub unsafe fn unmap_page(virt_addr: VirtAddr) {
    let page = Page::from_start_address(virt_addr).expect("unmap_page should be called with aligned addresses");
    unsafe { kernel_info().page_table.lock().unmap(page) };
}

#[doc(alias = "alloc::sync::Arc")]
#[repr(transparent)]
#[derive(Debug)]
pub struct Arc<T, A = alloc::alloc::Global>(alloc::sync::Arc<T, A>)
where
    T: ?Sized,
    A: alloc::alloc::Allocator;

impl<T, A> Arc<T, A>
where
    A: alloc::alloc::Allocator,
{
    pub fn new_in(data: T, alloc: A) -> Self {
        Self(alloc::sync::Arc::new_in(data, alloc))
    }
}

impl<T: ?Sized, A: alloc::alloc::Allocator + Clone> Clone for Arc<T, A> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> Arc<T, alloc::alloc::Global>
{
    pub fn new(data: T) -> Self {
        Self(alloc::sync::Arc::new(data))
    }
}

impl<T, A> core::ops::Deref for Arc<T, A>
where
    T: ?Sized,
    A: alloc::alloc::Allocator,
{
    type Target = T;

    fn deref(&self) -> &T {
        self.0.deref()
    }
}

impl<T: ?Sized + Unsize<U>, U: ?Sized, A: alloc::alloc::Allocator> CoerceUnsized<Arc<U, A>> for Arc<T, A> {}
impl<T: ?Sized + Unsize<U>, U: ?Sized> DispatchFromDyn<Arc<U>> for Arc<T> {}
