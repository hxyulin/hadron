use alloc::vec::Vec;

use crate::{
    arch::{PhysAddr, VirtAddr, registers::control::Cr3Flags},
    boot::frame_allocator::BootstrapFrameAllocator,
    mm::{
        allocator::{Locked, bump::BumpAllocator},
        mappings,
        page_table::{KernelPageTable, PageTable, PageTableEntry, PageTableFlags},
        paging::{PhysFrame, Size2MiB},
    },
};

#[derive(Debug, Clone, Copy)]
struct PdptTable {
    frame: PhysFrame,
    addr: VirtAddr,
    pml4_index: usize,
}

#[derive(Debug, Clone, Copy)]
struct PdTable {
    frame: PhysFrame,
    addr: VirtAddr,
    pml4_index: usize,
    pdpt_index: usize,
}

#[derive(Debug, Clone, Copy)]
struct PtTable {
    frame: PhysFrame,
    addr: VirtAddr,
    pml4_index: usize,
    pdpt_index: usize,
    pd_index: usize,
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

#[derive(Debug)]
pub struct BootstrapPageTable<'a> {
    pml4_phys: PhysFrame,
    pdpts: Vec<PdptTable, &'a Locked<BumpAllocator>>,
    pds: Vec<PdTable, &'a Locked<BumpAllocator>>,
    pts: Vec<PtTable, &'a Locked<BumpAllocator>>,
    hhdm_offset: usize,
}

impl<'a> BootstrapPageTable<'a> {
    pub fn new(
        hhdm_offset: usize,
        frame_allocator: &mut BootstrapFrameAllocator,
        allocator: &'a Locked<BumpAllocator>,
    ) -> Self {
        let pml4_phys = frame_allocator
            .allocate_mapped_frame()
            .expect("Failed to allocate frame");
        let pml4_addr = VirtAddr::new(pml4_phys.start_address().as_usize() + hhdm_offset);
        unsafe { pml4_addr.as_mut_ptr::<PageTable>().write(PageTable::new()) };
        let table = Self {
            pml4_phys,
            pdpts: Vec::new_in(allocator),
            pds: Vec::new_in(allocator),
            pts: Vec::new_in(allocator),
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
        unsafe { crate::arch::registers::control::Cr3::write(self.pml4_phys, Cr3Flags::empty()) };
    }

    /// Consuming self, returning the physical address of the page table
    pub fn as_phys_addr(self) -> PhysAddr {
        self.pml4_phys.start_address()
    }

    fn get_pml4(&mut self) -> &mut PageTable {
        unsafe {
            &mut *(VirtAddr::new(self.pml4_phys.start_address().as_usize() + self.hhdm_offset))
                .as_mut_ptr::<PageTable>()
        }
    }

    fn try_get_pdpt(&self, pml4_index: usize) -> Option<PdptTable> {
        debug_assert!(pml4_index != 510, "Cannot use recursive memory region");
        self.pdpts
            .iter()
            .find_map(|p| if p.pml4_index == pml4_index { Some(*p) } else { None })
    }

    fn get_or_create_pdpt(&mut self, pml4_index: usize, frame_allocator: &mut BootstrapFrameAllocator) -> PdptTable {
        self.try_get_pdpt(pml4_index).unwrap_or_else(|| {
            let frame = frame_allocator.allocate_mapped_frame().unwrap();
            let addr = VirtAddr::new(frame.start_address().as_usize() + self.hhdm_offset);
            unsafe { addr.as_mut_ptr::<PageTable>().write(PageTable::new()) };
            let table = PdptTable {
                frame,
                addr,
                pml4_index,
            };
            self.pdpts.push(table);
            let page_table = self.get_pml4();
            let mut entry = PageTableEntry::new();
            entry.set_frame(table.frame, PageTableFlags::PRESENT | PageTableFlags::WRITABLE);
            page_table[pml4_index as usize] = entry;

            table
        })
    }

    fn try_get_pd(&self, pml4_index: usize, pdpt_index: usize) -> Option<PdTable> {
        self.pds.iter().find_map(|p| {
            if p.pml4_index == pml4_index && p.pdpt_index == pdpt_index {
                Some(*p)
            } else {
                None
            }
        })
    }

    fn get_or_create_pd(
        &mut self,
        pml4_index: usize,
        pdpt_index: usize,
        frame_allocator: &mut BootstrapFrameAllocator,
    ) -> PdTable {
        self.try_get_pd(pml4_index, pdpt_index).unwrap_or_else(|| {
            let frame = frame_allocator.allocate_mapped_frame().unwrap();
            let addr = VirtAddr::new(frame.start_address().as_usize() + self.hhdm_offset);
            unsafe { addr.as_mut_ptr::<PageTable>().write(PageTable::new()) };
            let table = PdTable {
                frame,
                addr,
                pml4_index,
                pdpt_index,
            };

            self.pds.push(table);
            let pdpt = self.try_get_pdpt(pml4_index).unwrap();
            let pdpt_table = self.get_table(&pdpt);
            let mut entry = PageTableEntry::new();

            entry.set_frame(table.frame, PageTableFlags::PRESENT | PageTableFlags::WRITABLE);
            pdpt_table[pdpt_index as usize] = entry;

            table
        })
    }

    fn try_get_pt(&self, pml4_index: usize, pdpt_index: usize, pd_index: usize) -> Option<PtTable> {
        self.pts.iter().find_map(|p| {
            if p.pml4_index == pml4_index && p.pdpt_index == pdpt_index && p.pd_index == pd_index {
                Some(*p)
            } else {
                None
            }
        })
    }

    fn get_or_create_pt(
        &mut self,
        pml4_index: usize,
        pdpt_index: usize,
        pd_index: usize,
        frame_allocator: &mut BootstrapFrameAllocator,
    ) -> PtTable {
        self.try_get_pt(pml4_index, pdpt_index, pd_index).unwrap_or_else(|| {
            let frame = frame_allocator.allocate_mapped_frame().unwrap();
            let addr = VirtAddr::new(frame.start_address().as_usize() + self.hhdm_offset);
            unsafe { addr.as_mut_ptr::<PageTable>().write(PageTable::new()) };
            let table = PtTable {
                frame,
                addr,
                pml4_index,
                pdpt_index,
                pd_index,
            };

            self.pts.push(table);
            let pd = self.try_get_pd(pml4_index, pdpt_index).unwrap();
            let pd_table = self.get_table(&pd);
            let mut entry = PageTableEntry::new();

            entry.set_frame(table.frame, PageTableFlags::PRESENT | PageTableFlags::WRITABLE);
            pd_table[pd_index as usize] = entry;
            table
        })
    }

    pub fn map(
        &mut self,
        addr: VirtAddr,
        frame: PhysFrame,
        flags: PageTableFlags,
        frame_allocator: &mut BootstrapFrameAllocator,
    ) {
        let pml4_index: usize = addr.p4_index();
        let _pdpt = self.get_or_create_pdpt(pml4_index, frame_allocator);

        let pdpt_index: usize = addr.p3_index();
        let _pd = self.get_or_create_pd(pml4_index, pdpt_index, frame_allocator);

        let pd_index: usize = addr.p2_index();
        let pt = self.get_or_create_pt(pml4_index, pdpt_index, pd_index, frame_allocator);

        let pt_table = self.get_table(&pt);
        let mut entry = PageTableEntry::new();

        entry.set_frame(frame, flags);
        pt_table[addr.p1_index()] = entry;
    }

    pub fn map_2mib(
        &mut self,
        addr: VirtAddr,
        frame: PhysFrame<Size2MiB>,
        flags: PageTableFlags,
        frame_allocator: &mut BootstrapFrameAllocator,
    ) {
        let pml4_index: usize = addr.p4_index();
        let _pdpt = self.get_or_create_pdpt(pml4_index, frame_allocator);

        let pdpt_index: usize = addr.p3_index();
        let pd = self.get_or_create_pd(pml4_index, pdpt_index, frame_allocator);

        let pd_table = self.get_table(&pd);
        let mut entry = PageTableEntry::new();

        entry.set_addr(frame.start_address(), flags);
        pd_table[addr.p2_index()] = entry;
    }

    /// Direct maps internally used page tables to their respective virtual addresses
    /// This is used for the creation of the Direct Mapped Kernel Page Table
    pub fn direct_map(&mut self, frame_allocator: &mut BootstrapFrameAllocator) {
        use crate::mm::page_table::KernelPageTable;

        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_EXECUTE;
        for idx in 0..self.pts.len() {
            let page = &self.pts[idx];
            let addr = KernelPageTable::DIRECT_MAP_START + page.frame.start_address().as_usize();
            self.map(addr, page.frame, flags, frame_allocator);
        }

        for idx in 0..self.pds.len() {
            let page = &self.pds[idx];
            let addr = KernelPageTable::DIRECT_MAP_START + page.frame.start_address().as_usize();
            self.map(addr, page.frame, flags, frame_allocator);
        }

        for idx in 0..self.pdpts.len() {
            let page = &self.pdpts[idx];
            let addr = KernelPageTable::DIRECT_MAP_START + page.frame.start_address().as_usize();
            self.map(addr, page.frame, flags, frame_allocator);
        }

        let addr = KernelPageTable::DIRECT_MAP_START + self.pml4_phys.start_address().as_usize();
        self.map(addr, self.pml4_phys, flags, frame_allocator);
    }
}
