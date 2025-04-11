use core::ops::DerefMut;

use x86_64::{
    VirtAddr,
    structures::paging::{Mapper, Page, PageTable, PageTableFlags, PhysFrame, RecursivePageTable},
};

use crate::base::info::kernel_info;

#[derive(Debug)]
pub struct KernelPageTable {
    table: RecursivePageTable<'static>,
}

impl KernelPageTable {
    pub fn new() -> Self {
        // 110 for each page table index
        let table_addr = VirtAddr::new(0xFFFF_FF7F_BFDF_E000);
        let table = unsafe { &mut *(table_addr.as_mut_ptr::<PageTable>()) };
        let table = RecursivePageTable::new(table).expect("failed to create recursive page table");
        Self { table }
    }

    pub unsafe fn map(&mut self, page: Page, frame: PhysFrame, flags: PageTableFlags) {
        let mut frame_alloc = kernel_info().frame_allocator.lock();
        unsafe { self.table.map_to(page, frame, flags, frame_alloc.deref_mut()) }
            .unwrap()
            .flush();
    }

    pub fn unmap(&mut self, page: Page) {
        self.table.unmap(page).unwrap().1.flush();
    }
}
