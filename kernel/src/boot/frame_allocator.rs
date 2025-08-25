use crate::{
    arch::PhysAddr,
    boot::memory_map::{BootstrapMemoryMap, MemoryMapEntry, MemoryRegionType},
    mm::paging::{FrameAllocator, PageSize, PhysFrame, Size4KiB},
};

pub struct BootstrapFrameAllocator<'a> {
    memory_map: &'a mut BootstrapMemoryMap,
}

impl<'a> BootstrapFrameAllocator<'a> {
    pub fn new(memory_map: &'a mut BootstrapMemoryMap) -> Self {
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

                return Some(PhysFrame::from_start_address(frame_addr));
            }

            i += 1;
        }
        None
    }

    /// Allocates a contiguous region of frames
    pub fn allocate_mapped_contiguous(&mut self, count: usize) -> Option<PhysFrame> {
        let size = count * Size4KiB::SIZE;
        let mut i = 0;
        while i < self.memory_map.len() {
            let region = &mut self.memory_map[i];

            // Skip regions above 4GiB
            if region.base().as_u64() >= 0x0001_0000_0000 {
                break;
            }

            if region.ty() == MemoryRegionType::Usable && region.length() >= size {
                let frame_addr = region.base();

                // Adjust the existing region instead of creating a new one
                region.base = frame_addr + size;
                region.length -= size;

                // If region is now empty, mark it as allocated
                if region.length() == 0 {
                    region.memory_type = MemoryRegionType::Allocated;
                }

                return Some(PhysFrame::from_start_address(frame_addr));
            }

            i += 1;
        }
        None
    }

    pub fn deallocate_region(&mut self, start: PhysAddr, length: usize) {
        assert!(length % Size4KiB::SIZE == 0);
        assert!(start.as_usize() % Size4KiB::SIZE == 0);
        self.memory_map.push(MemoryMapEntry {
            base: start,
            length,
            memory_type: MemoryRegionType::Usable,
        });
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootstrapFrameAllocator<'_> {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
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

                return Some(PhysFrame::from_start_address(frame_addr));
            }

            i += 1;
        }
        None
    }
}
