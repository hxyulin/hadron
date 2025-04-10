use x86_64::{
    VirtAddr,
    structures::paging::{PageTable, RecursivePageTable},
};

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
}
