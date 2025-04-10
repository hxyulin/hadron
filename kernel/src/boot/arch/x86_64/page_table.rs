use x86_64::{
    PhysAddr, VirtAddr,
    registers::control::Cr3Flags,
    structures::paging::{FrameDeallocator, PageTable, PageTableFlags, PhysFrame, page_table::PageTableEntry},
};

use super::frame_allocator::BasicFrameAllocator;

#[derive(Debug, Clone, Copy)]
struct PdptTable {
    frame: PhysFrame,
    addr: VirtAddr,
    pml4_index: u16,
}

#[derive(Debug, Clone, Copy)]
struct PdTable {
    frame: PhysFrame,
    addr: VirtAddr,
    pml4_index: u16,
    pdpt_index: u16,
}

#[derive(Debug, Clone, Copy)]
struct PtTable {
    frame: PhysFrame,
    addr: VirtAddr,
    pml4_index: u16,
    pdpt_index: u16,
    pd_index: u16,
}

#[allow(dead_code)]
trait PageSubTable {
    fn get_frame(&self) -> PhysFrame;
    fn get_addr(&self) -> VirtAddr;
}

impl PageSubTable for PdptTable {
    fn get_frame(&self) -> PhysFrame {
        self.frame
    }

    fn get_addr(&self) -> VirtAddr {
        self.addr
    }
}

impl PageSubTable for PdTable {
    fn get_frame(&self) -> PhysFrame {
        self.frame
    }

    fn get_addr(&self) -> VirtAddr {
        self.addr
    }
}

impl PageSubTable for PtTable {
    fn get_frame(&self) -> PhysFrame {
        self.frame
    }
    fn get_addr(&self) -> VirtAddr {
        self.addr
    }
}

pub struct BootstrapPageTable {
    pml4_phys: PhysFrame,
    pdpts: [Option<PdptTable>; 8],
    pds: [Option<PdTable>; 8],
    pt: [Option<PtTable>; 8],
    hhdm_offset: u64,
}

impl BootstrapPageTable {
    pub fn new(hhdm_offset: u64, frame_allocator: &mut BasicFrameAllocator) -> Self {
        let pml4_phys = frame_allocator
            .allocate_mapped_frame()
            .expect("Failed to allocate frame");
        let pml4_addr = VirtAddr::new(pml4_phys.start_address().as_u64() + hhdm_offset);
        unsafe { pml4_addr.as_mut_ptr::<PageTable>().write(PageTable::new()) };
        let table = Self {
            pml4_phys,
            pdpts: [None; 8],
            pds: [None; 8],
            pt: [None; 8],
            hhdm_offset,
        };

        // We need to recursive map the page tables
        let pml4_table = unsafe { &mut *(pml4_addr.as_mut_ptr::<PageTable>()) };
        let mut entry = PageTableEntry::new();
        entry.set_frame(table.pml4_phys, PageTableFlags::PRESENT | PageTableFlags::WRITABLE);
        pml4_table[510] = entry;

        table
    }

    #[inline]
    fn get_table(&mut self, table: &impl PageSubTable) -> &mut PageTable {
        unsafe { &mut *(table.get_addr().as_mut_ptr()) }
    }

    pub fn load(&self) {
        unsafe { x86_64::registers::control::Cr3::write(self.pml4_phys, Cr3Flags::empty()) };
    }

    pub fn phys_addr(&self) -> PhysAddr {
        self.pml4_phys.start_address()
    }

    fn get_pml4(&mut self) -> &mut PageTable {
        unsafe {
            &mut *(VirtAddr::new(self.pml4_phys.start_address().as_u64() + self.hhdm_offset)).as_mut_ptr::<PageTable>()
        }
    }

    fn try_get_pdpt(&self, pml4_index: u16) -> Option<PdptTable> {
        debug_assert!(pml4_index != 510, "Cannot use recursive memory region");
        self.pdpts.iter().find_map(|p| {
            if let Some(p) = p {
                if p.pml4_index == pml4_index { Some(*p) } else { None }
            } else {
                None
            }
        })
    }

    fn get_or_create_pdpt(&mut self, pml4_index: u16, frame_allocator: &mut BasicFrameAllocator) -> PdptTable {
        self.try_get_pdpt(pml4_index).unwrap_or_else(|| {
            let frame = frame_allocator.allocate_mapped_frame().unwrap();
            let addr = VirtAddr::new(frame.start_address().as_u64() + self.hhdm_offset);
            unsafe { addr.as_mut_ptr::<PageTable>().write(PageTable::new()) };
            let table = PdptTable {
                frame,
                addr,
                pml4_index,
            };
            // Find first free entry
            for i in 0..8 {
                if self.pdpts[i].is_none() {
                    self.pdpts[i] = Some(table);
                    let page_table = self.get_pml4();
                    let mut entry = PageTableEntry::new();
                    entry.set_addr(
                        table.frame.start_address(),
                        PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
                    );
                    page_table[pml4_index as usize] = entry;

                    // Now we need to recursive map the page tables

                    return table;
                }
            }
            // TODO: Unallocate, and maybe make it not unrecoverable
            panic!("Too many PDPTs");
        })
    }

    fn try_get_pd(&self, pml4_index: u16, pdpt_index: u16) -> Option<PdTable> {
        self.pds.iter().find_map(|p| {
            if let Some(p) = p {
                if p.pml4_index == pml4_index && p.pdpt_index == pdpt_index {
                    return Some(*p);
                }
            }
            None
        })
    }

    fn get_or_create_pd(
        &mut self,
        pml4_index: u16,
        pdpt_index: u16,
        frame_allocator: &mut BasicFrameAllocator,
    ) -> PdTable {
        self.try_get_pd(pml4_index, pdpt_index).unwrap_or_else(|| {
            let frame = frame_allocator.allocate_mapped_frame().unwrap();
            let addr = VirtAddr::new(frame.start_address().as_u64() + self.hhdm_offset);
            unsafe { addr.as_mut_ptr::<PageTable>().write(PageTable::new()) };
            let table = PdTable {
                frame,
                addr,
                pml4_index,
                pdpt_index,
            };

            // Find first free entry
            for i in 0..8 {
                if self.pds[i].is_none() {
                    self.pds[i] = Some(table);
                    let pdpt = self.try_get_pdpt(pml4_index).unwrap();
                    let pdpt_table = self.get_table(&pdpt);
                    let mut entry = PageTableEntry::new();
                    entry.set_frame(table.frame, PageTableFlags::PRESENT | PageTableFlags::WRITABLE);
                    pdpt_table[pdpt_index as usize] = entry;

                    return table;
                }
            }
            panic!("Too many PDs");
        })
    }

    fn try_get_pt(&self, pml4_index: u16, pdpt_index: u16, pd_index: u16) -> Option<PtTable> {
        self.pt.iter().find_map(|p| {
            if let Some(p) = p {
                if p.pml4_index == pml4_index && p.pdpt_index == pdpt_index && p.pd_index == pd_index {
                    return Some(*p);
                }
            }
            None
        })
    }

    fn get_or_create_pt(
        &mut self,
        pml4_index: u16,
        pdpt_index: u16,
        pd_index: u16,
        frame_allocator: &mut BasicFrameAllocator,
    ) -> PtTable {
        self.try_get_pt(pml4_index, pdpt_index, pd_index).unwrap_or_else(|| {
            let frame = frame_allocator.allocate_mapped_frame().unwrap();
            let addr = VirtAddr::new(frame.start_address().as_u64() + self.hhdm_offset);
            unsafe { addr.as_mut_ptr::<PageTable>().write(PageTable::new()) };
            let table = PtTable {
                frame,
                addr,
                pml4_index,
                pdpt_index,
                pd_index,
            };

            for i in 0..8 {
                if self.pt[i].is_none() {
                    self.pt[i] = Some(table);
                    let pd = self.try_get_pd(pml4_index, pdpt_index).unwrap();
                    let pd_table = self.get_table(&pd);
                    let mut entry = PageTableEntry::new();
                    entry.set_frame(table.frame, PageTableFlags::PRESENT | PageTableFlags::WRITABLE);
                    pd_table[pd_index as usize] = entry;
                    return table;
                }
            }
            panic!("Too many PTs");
        })
    }

    pub fn map(
        &mut self,
        addr: VirtAddr,
        frame: PhysFrame,
        flags: PageTableFlags,
        frame_allocator: &mut BasicFrameAllocator,
    ) {
        let pml4_index: u16 = addr.p4_index().into();
        let _pdpt = self.get_or_create_pdpt(pml4_index, frame_allocator);

        let pdpt_index: u16 = addr.p3_index().into();
        let _pd = self.get_or_create_pd(pml4_index, pdpt_index, frame_allocator);

        let pd_index: u16 = addr.p2_index().into();
        let pt = self.get_or_create_pt(pml4_index, pdpt_index, pd_index, frame_allocator);

        let pt_table = self.get_table(&pt);
        let mut entry = PageTableEntry::new();
        entry.set_frame(frame, flags);
        pt_table[addr.p1_index()] = entry;
    }

    /// # Safety
    /// This function is unsafe because it deallocates frames without checking if they are still
    /// in use.
    ///
    /// This function should never be called if the page table is still in use (entire duration of
    /// the kernel).
    pub unsafe fn deinit(&mut self, frame_allocator: &mut BasicFrameAllocator) {
        for pdpt in self.pdpts.iter_mut().flatten() {
            unsafe { frame_allocator.deallocate_frame(pdpt.frame) };
        }
        for pd in self.pds.iter_mut().flatten() {
            unsafe { frame_allocator.deallocate_frame(pd.frame) };
        }
        for pt in self.pt.iter_mut().flatten() {
            unsafe { frame_allocator.deallocate_frame(pt.frame) };
        }
    }
}
