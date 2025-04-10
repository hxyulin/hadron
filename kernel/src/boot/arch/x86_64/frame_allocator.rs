use x86_64::structures::paging::{FrameAllocator, FrameDeallocator, PageSize, PhysFrame, Size4KiB};

use core::option::Option;

use crate::boot::arch::memory_map::{MemoryMap, MemoryMapEntry, MemoryRegionType};

pub struct BasicFrameAllocator<'ctx> {
    memory_map: &'ctx mut MemoryMap,
}

impl<'ctx> BasicFrameAllocator<'ctx> {
    pub fn new(memory_map: &'ctx mut MemoryMap) -> Self {
        Self { memory_map }
    }

    pub fn allocate_mapped_frame(&mut self) -> Option<PhysFrame> {
        // Keep track of which region we last allocated from to avoid rescanning
        let mut i = 0;
        while i < self.memory_map.size as usize {
            let region = &mut self.memory_map.entries[i];

            // Skip regions above 4GiB
            if region.base().as_u64() >= 0x0001_0000_0000 {
                break;
            }

            if region.ty() == MemoryRegionType::Usable && region.length() >= Size4KiB::SIZE {
                let frame_addr = region.base();

                // Adjust the existing region instead of creating a new one
                region.base = frame_addr + Size4KiB::SIZE;
                region.length -= Size4KiB::SIZE;

                // If region is now empty, mark it as allocated
                if region.length() == 0 {
                    region.memory_type = MemoryRegionType::Allocated;
                }

                return Some(PhysFrame::containing_address(frame_addr));
            }

            i += 1;
        }
        None
    }

    /// Allocates a contiguous region of frames, returning the first frame.
    pub fn allocate_contiguous(&mut self, count: u64) -> Option<PhysFrame> {
        let total_size = count * Size4KiB::SIZE;

        let mut i = 0;
        while i < self.memory_map.size as usize {
            let region = &mut self.memory_map.entries[i];

            if region.ty() == MemoryRegionType::Usable && region.length() >= total_size {
                let frame_addr = region.base();

                // Adjust the existing region instead of creating a new one
                region.base = frame_addr + total_size;
                region.length -= total_size;

                // If region is now empty, mark it as allocated
                if region.length() == 0 {
                    region.memory_type = MemoryRegionType::Allocated;
                }

                return Some(PhysFrame::containing_address(frame_addr));
            }

            i += 1;
        }
        None
    }
}

unsafe impl FrameAllocator<x86_64::structures::paging::Size4KiB> for BasicFrameAllocator<'_> {
    fn allocate_frame(&mut self) -> Option<x86_64::structures::paging::PhysFrame> {
        let mut i = 0;
        while i < self.memory_map.size as usize {
            let region = &mut self.memory_map.entries[i];

            if region.ty() == MemoryRegionType::Usable && region.length() >= Size4KiB::SIZE {
                let frame_addr = region.base();

                // Adjust the existing region instead of creating a new one
                region.base = frame_addr + Size4KiB::SIZE;
                region.length -= Size4KiB::SIZE;

                // If region is now empty, mark it as allocated
                if region.length() == 0 {
                    region.memory_type = MemoryRegionType::Allocated;
                }

                return Some(PhysFrame::containing_address(frame_addr));
            }

            i += 1;
        }
        None
    }
}

impl FrameDeallocator<x86_64::structures::paging::Size4KiB> for BasicFrameAllocator<'_> {
    unsafe fn deallocate_frame(&mut self, frame: x86_64::structures::paging::PhysFrame) {
        let frame_addr = frame.start_address();
        let memory_region = MemoryMapEntry {
            base: frame_addr,
            length: Size4KiB::SIZE,
            memory_type: MemoryRegionType::Allocated,
        };
        // We don't really care about defragmentation, because we will immediately clean this up
        // after we setup the page table with the heap
        self.memory_map.entries[self.memory_map.size as usize] = memory_region;
        self.memory_map.size += 1;
    }
}
