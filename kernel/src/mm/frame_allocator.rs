use crate::mm::paging::{FrameAllocator, FrameDeallocator, PageSize, PhysFrame, Size4KiB};

use super::memory_map::{MemoryMap, MemoryRegion, MemoryRegionTag};

#[derive(Debug)]
pub struct KernelFrameAllocator {
    memory_map: MemoryMap,
}

impl KernelFrameAllocator {
    pub const fn new(memory_map: MemoryMap) -> Self {
        Self { memory_map }
    }

    pub fn free_special_region(&mut self, tag: MemoryRegionTag) {
        let allocator = self.memory_map.alloc.clone();
        self.memory_map.special.retain(|entry| {
            if entry.tag == tag {
                self.memory_map.entries.push(MemoryRegion::from_base_and_length(
                    entry.base,
                    entry.length,
                    allocator.clone(),
                ));
                false
            } else {
                true
            }
        });
    }

    /// Returns the total amount of memory in the system (in pages)
    pub fn total_pages(&self) -> usize {
        self.memory_map.entries.iter().map(|entry| entry.pages()).sum()
    }

    pub fn memory_map(&self) -> &MemoryMap {
        &self.memory_map
    }

    /// Returns a mutable reference to the underlying memory map
    ///
    /// # Safety
    /// This is unsafe because the caller must ensure that the memory map is not modified such that
    /// previous allocations are invalidated
    pub unsafe fn memory_map_mut(&mut self) -> &mut MemoryMap {
        &mut self.memory_map
    }
}

unsafe impl FrameAllocator<Size4KiB> for KernelFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        for entry in &mut self.memory_map.entries {
            if let Some(idx) = entry.allocate() {
                return Some(PhysFrame::from_start_address(entry.base + idx * Size4KiB::SIZE));
            }
        }
        None
    }
}

impl FrameDeallocator<Size4KiB> for KernelFrameAllocator {
    unsafe fn deallocate_frame(&mut self, frame: PhysFrame) {
        for entry in &mut self.memory_map.entries {
            if !entry.contains(frame.start_address()) {
                continue;
            }

            let idx = (frame.start_address().as_usize() - entry.base.as_usize()) / Size4KiB::SIZE;
            entry.deallocate(idx);
        }
        panic!("Deallocating frame that is not allocated");
    }
}
