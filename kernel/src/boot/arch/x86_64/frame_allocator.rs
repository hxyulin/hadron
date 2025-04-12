use x86_64::structures::paging::{FrameAllocator, FrameDeallocator, PageSize, PhysFrame, Size4KiB};

use core::option::Option;

use crate::boot::arch::memory_map::{BootstrapMemoryMap, MemoryMapEntry, MemoryRegionType};

#[derive(Debug)]
pub struct BasicFrameAllocator<'ctx> {
    memory_map: &'ctx mut BootstrapMemoryMap,
}

impl<'ctx> BasicFrameAllocator<'ctx> {
    pub fn new(memory_map: &'ctx mut BootstrapMemoryMap) -> Self {
        Self { memory_map }
    }

    pub fn allocate_mapped_frame(&mut self) -> Option<PhysFrame> {
        // Keep track of which region we last allocated from to avoid rescanning
        let mut i = 0;
        while i < self.memory_map.len() {
            let region = &mut self.memory_map[i];

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

    /// Allocates a contiguous region of frames
    pub fn allocate_mapped_contiguous(&mut self, count: usize) -> Option<PhysFrame> {
        let size = count as u64 * Size4KiB::SIZE;
        let mut i = 0;
        while i < self.memory_map.len() {
            let region = &mut self.memory_map[i];

            // Skip regions above 4GiB
            if region.base().as_u64() >= 0x0001_0000_0000 {
                break;
            }

            if region.ty() == MemoryRegionType::Usable && region.length() >=size {
                let frame_addr = region.base();

                // Adjust the existing region instead of creating a new one
                region.base = frame_addr + size;
                region.length -= size;

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
        while i < self.memory_map.len() {
            let region = &mut self.memory_map[i];

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
        unimplemented!()
    }
}
