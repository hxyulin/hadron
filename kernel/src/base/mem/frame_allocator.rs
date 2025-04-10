use x86_64::structures::paging::{FrameAllocator, FrameDeallocator, PageSize, PhysFrame, Size4KiB};

use super::memory_map::{MemoryMap, MemoryRegion, MemoryRegionTag};

#[derive(Debug)]
pub struct KernelFrameAllocator {
    memory_map: MemoryMap,
}

impl KernelFrameAllocator {
    pub fn new(memory_map: MemoryMap) -> Self {
        Self { memory_map }
    }

    pub fn free_special_region(&mut self, tag: MemoryRegionTag) {
        self.memory_map.special.retain(|entry| {
            if entry.tag == tag {
                self.memory_map
                    .entries
                    .push(MemoryRegion::from_base_and_length(entry.base, entry.length));
                false
            } else {
                true
            }
        });
    }

    /// Returns the total amount of memory in the system (in pages)
    pub fn total_pages(&self) -> u64 {
        self.memory_map.entries.iter().map(|entry| entry.pages).sum()
    }
}

unsafe impl FrameAllocator<Size4KiB> for KernelFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        for entry in &mut self.memory_map.entries {
            if let Some(idx) = entry.allocate() {
                return Some(PhysFrame::containing_address(entry.base + idx as u64 * Size4KiB::SIZE));
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

            let idx = (frame.start_address().as_u64() - entry.base.as_u64()) / Size4KiB::SIZE;
            entry.deallocate(idx as usize);
        }
        panic!("Deallocating frame that is not allocated");
    }
}
