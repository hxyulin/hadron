use crate::{mm::frame_allocator::KernelFrameAllocator, sync::mutex::UninitMutex};

pub mod allocator;
pub mod frame_allocator;
pub mod mappings;
pub mod memory_map;
pub mod page_table;
pub mod paging;

pub static FRAME_ALLOCATOR: UninitMutex<KernelFrameAllocator> = UninitMutex::<KernelFrameAllocator>::uninit();
