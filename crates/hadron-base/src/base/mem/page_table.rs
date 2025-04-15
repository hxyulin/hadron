use core::ops::DerefMut;

use x86_64::structures::paging::{FrameAllocator, PageSize, RecursivePageTable, Size2MiB, Size4KiB};
use x86_64::{
    VirtAddr,
    structures::paging::{Mapper, Page, PageTableFlags, PhysFrame},
};

use super::FRAME_ALLOCATOR;

#[derive(Debug)]
pub struct KernelPageTable {
    table: RecursivePageTable<'static>,
}

impl KernelPageTable {
    pub fn new() -> Self {
        use x86_64::structures::paging::PageTable;
        // 510 for each page table index
        let table_addr = VirtAddr::new(0xFFFF_FF7F_BFDF_E000);
        let table = unsafe { &mut *(table_addr.as_mut_ptr::<PageTable>()) };
        let table = RecursivePageTable::new(table).expect("failed to create recursive page table");
        Self { table }
    }
}

pub trait PageTable<S: PageSize = Size4KiB> {
    /// Maps a page to a frame.
    ///
    /// # Safety
    /// This function is unsafe because mapping the same physical frame to multiple virtual addresses
    /// can cause UB.
    unsafe fn map(&mut self, page: Page<S>, frame: PhysFrame<S>, flags: PageTableFlags);
    unsafe fn map_with_allocator(
        &mut self,
        page: Page<S>,
        frame: PhysFrame<S>,
        flags: PageTableFlags,
        frame_alloc: &mut impl FrameAllocator<Size4KiB>,
    );
    unsafe fn unmap(&mut self, page: Page<S>);
}

impl PageTable<Size4KiB> for KernelPageTable {
    unsafe fn map(&mut self, page: Page<Size4KiB>, frame: PhysFrame<Size4KiB>, flags: PageTableFlags) {
        let mut frame_alloc = FRAME_ALLOCATOR.lock();
        unsafe { self.map_with_allocator(page, frame, flags, frame_alloc.deref_mut()) };
    }

    unsafe fn map_with_allocator(
        &mut self,
        page: Page<Size4KiB>,
        frame: PhysFrame<Size4KiB>,
        flags: PageTableFlags,
        frame_alloc: &mut impl FrameAllocator<Size4KiB>,
    ) {
        unsafe { self.table.map_to(page, frame, flags, frame_alloc) }
            .expect("failed to map page to frame")
            .flush();
    }

    unsafe fn unmap(&mut self, page: Page) {
        self.table.unmap(page).unwrap().1.flush();
    }
}

impl PageTable<Size2MiB> for KernelPageTable {
    unsafe fn map(&mut self, _page: Page<Size2MiB>, _frame: PhysFrame<Size2MiB>, _flags: PageTableFlags) {
        unimplemented!("cannot directly map 2MiB pages, use 4KiB pages instead")
    }

    unsafe fn map_with_allocator(
        &mut self,
        page: Page<Size2MiB>,
        frame: PhysFrame<Size2MiB>,
        flags: PageTableFlags,
        frame_alloc: &mut impl FrameAllocator<Size4KiB>,
    ) {
        unsafe { self.table.map_to(page, frame, flags, frame_alloc) }
            .expect("failed to map page to frame")
            .flush();
    }

    unsafe fn unmap(&mut self, page: Page<Size2MiB>) {
        self.table.unmap(page).unwrap().1.flush();
    }
}
