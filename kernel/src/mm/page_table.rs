use core::{
    fmt,
    ops::{Index, IndexMut},
};

#[cfg(target_arch = "x86_64")]
use crate::arch::instructions::invlpg;
use crate::{
    arch::{PhysAddr, VirtAddr},
    kprintln,
    mm::{
        mappings,
        paging::{FrameAllocator, Page, PageSize, PhysFrame, Size4KiB},
    },
};

#[derive(Debug)]
#[repr(C, align(4096))]
pub struct PageTable {
    pub entries: [PageTableEntry; 512],
}

impl PageTable {
    pub const fn new() -> Self {
        Self {
            entries: [const { PageTableEntry::new() }; 512],
        }
    }
}

impl Index<usize> for PageTable {
    type Output = PageTableEntry;

    fn index(&self, index: usize) -> &Self::Output {
        &self.entries[index]
    }
}

impl IndexMut<usize> for PageTable {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.entries[index]
    }
}

#[repr(transparent)]
pub struct PageTableEntry {
    entry: u64,
}

impl PageTableEntry {
    const PHYS_ADDR_MASK: u64 = 0x000f_ffff_ffff_f000u64;

    pub const fn new() -> Self {
        Self { entry: 0 }
    }

    pub fn addr(&self) -> PhysAddr {
        PhysAddr::new((self.entry & Self::PHYS_ADDR_MASK) as usize)
    }

    pub fn set_addr(&mut self, addr: PhysAddr, flags: PageTableFlags) {
        assert!(addr.is_aligned(Size4KiB::SIZE), "page address is not page aligned");
        self.entry = addr.as_u64() | flags.bits();
    }

    pub fn flags(&self) -> PageTableFlags {
        PageTableFlags::from_bits_truncate(self.entry & !Self::PHYS_ADDR_MASK)
    }

    pub fn set_flags(&mut self, flags: PageTableFlags) {
        self.entry = self.addr().as_u64() | flags.bits();
    }

    pub fn set_frame(&mut self, frame: PhysFrame, flags: PageTableFlags) {
        self.entry = frame.start_address().as_u64() | flags.bits()
    }

    pub fn is_present(&self) -> bool {
        self.flags().contains(PageTableFlags::PRESENT)
    }
}

impl fmt::Debug for PageTableEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PageTableEntry")
            .field("addr", &self.addr())
            .field("flags", &self.flags())
            .finish()
    }
}

/// THe Page Table for the Kernel
///
/// The Page Table is implemented using the Direct Mapping Strategy
pub struct KernelPageTable {
    pml4: VirtAddr,
}

impl KernelPageTable {
    pub const DIRECT_MAP_START: VirtAddr = mappings::PAGE_TABLE_START;
    pub const DIRECT_MAP_OFFSET: usize = Self::DIRECT_MAP_START.as_usize();
    pub fn new(pml4: PhysAddr) -> Self {
        Self {
            pml4: Self::DIRECT_MAP_START + pml4.as_usize(),
        }
    }
}

#[cfg(target_arch = "x86_64")]
impl KernelPageTable {
    const PAGE_TABLE_FLAGS: PageTableFlags = PageTableFlags::from_bits_truncate(
        PageTableFlags::PRESENT.bits() | PageTableFlags::WRITABLE.bits() | PageTableFlags::NO_EXECUTE.bits(),
    );

    fn to_pt<'a>(addr: VirtAddr) -> &'a PageTable {
        unsafe { addr.as_ptr::<PageTable>().as_ref().expect("pml4 is null!") }
    }

    fn to_pt_mut<'a>(addr: VirtAddr) -> &'a mut PageTable {
        unsafe { addr.as_mut_ptr::<PageTable>().as_mut().expect("pml4 is null!") }
    }

    fn pml4(&self) -> &PageTable {
        Self::to_pt(self.pml4)
    }

    fn pml4_mut(&mut self) -> &mut PageTable {
        Self::to_pt_mut(self.pml4)
    }

    fn get_or_create_pdpt(&mut self, index: usize, alloc: &mut impl FrameAllocator<Size4KiB>) -> &mut PageTable {
        let entry = &mut self.pml4_mut()[index];
        if entry.is_present() {
            return Self::to_pt_mut(Self::DIRECT_MAP_START + entry.addr().as_usize());
        }
        let frame = alloc.allocate_frame().expect("no frames to allocate");
        let virt = Self::DIRECT_MAP_START + frame.start_address().as_usize();
        entry.set_frame(frame, Self::PAGE_TABLE_FLAGS);
        unsafe {
            self.map_with_allocator(
                Page::<Size4KiB>::from_start_address(virt),
                frame,
                Self::PAGE_TABLE_FLAGS,
                alloc,
            )
        };
        unsafe {
            invlpg(virt);
        }
        let new_pt = Self::to_pt_mut(virt);
        *new_pt = PageTable::new();
        new_pt
    }

    fn get_or_create_pd(
        &mut self,
        pdpt_index: usize,
        index: usize,
        alloc: &mut impl FrameAllocator<Size4KiB>,
    ) -> &mut PageTable {
        let entry = &mut self.get_or_create_pdpt(pdpt_index, alloc)[index];
        if entry.is_present() {
            return Self::to_pt_mut(Self::DIRECT_MAP_START + entry.addr().as_usize());
        }
        let frame = alloc.allocate_frame().expect("no frames to allocate");
        let virt = Self::DIRECT_MAP_START + frame.start_address().as_usize();
        entry.set_frame(frame, Self::PAGE_TABLE_FLAGS);
        unsafe {
            invlpg(virt);
        }
        let new_pt = Self::to_pt_mut(virt);
        *new_pt = PageTable::new();
        new_pt
    }

    fn get_or_create_pt(
        &mut self,
        pdpt_index: usize,
        pd_index: usize,
        index: usize,
        alloc: &mut impl FrameAllocator<Size4KiB>,
    ) -> &mut PageTable {
        let entry = &mut self.get_or_create_pd(pdpt_index, pd_index, alloc)[index];
        if entry.is_present() {
            return Self::to_pt_mut(Self::DIRECT_MAP_START + entry.addr().as_usize());
        }
        let frame = alloc.allocate_frame().expect("no frames to allocate");
        let virt = Self::DIRECT_MAP_START + frame.start_address().as_usize();
        entry.set_frame(frame, Self::PAGE_TABLE_FLAGS);
        unsafe {
            invlpg(virt);
        }
        let new_pt = Self::to_pt_mut(virt);
        *new_pt = PageTable::new();
        new_pt
    }

    pub fn dump(&self) {
        for (pml4_idx, pml4_entry) in self.pml4().entries.iter().enumerate() {
            if !pml4_entry.is_present() {
                continue;
            }

            kprintln!(Debug, "PML4[{}]: {:?}", pml4_idx, pml4_entry);

            let pdpt_virt = Self::DIRECT_MAP_START + pml4_entry.addr().as_usize();
            let pdpt = Self::to_pt(pdpt_virt);

            for (pdpt_idx, pdpt_entry) in pdpt.entries.iter().enumerate() {
                if !pdpt_entry.is_present() {
                    continue;
                }

                kprintln!(Debug, "\tPDPT[{}]: {:?}", pdpt_idx, pdpt_entry);

                // Check if this is a 1GB page
                if pdpt_entry.flags().contains(PageTableFlags::HUGE_PAGE) {
                    kprintln!(Debug, "\t\t1GB PAGE - Skipping further traversal");
                    continue;
                }

                let pd_virt = Self::DIRECT_MAP_START + pdpt_entry.addr().as_usize();
                let pd = Self::to_pt(pd_virt);

                for (pd_idx, pd_entry) in pd.entries.iter().enumerate() {
                    if !pd_entry.is_present() {
                        continue;
                    }

                    kprintln!(Debug, "\t\tPD[{}]: {:?}", pd_idx, pd_entry);

                    // Check if this is a 2MB page
                    if pd_entry.flags().contains(PageTableFlags::HUGE_PAGE) {
                        kprintln!(Debug, "\t\t\t2MB PAGE - Skipping further traversal");
                        continue;
                    }

                    let pt_virt = Self::DIRECT_MAP_START + pd_entry.addr().as_usize();
                    let pt = Self::to_pt(pt_virt);

                    for (pt_idx, pt_entry) in pt.entries.iter().enumerate() {
                        if !pt_entry.is_present() {
                            continue;
                        }

                        kprintln!(Debug, "\t\t\tPT[{}]: {:?}", pt_idx, pt_entry);

                        // Calculate the virtual address this entry maps to
                        let virt_addr = VirtAddr::new_truncate(
                            (pml4_idx << 39) | (pdpt_idx << 30) | (pd_idx << 21) | (pt_idx << 12),
                        );

                        kprintln!(
                            Debug,
                            "\t\t\t\tMaps virtual address: {:?} to physical: {:?}",
                            virt_addr,
                            pt_entry.addr()
                        );
                    }
                }
            }
        }
    }
}

impl fmt::Debug for KernelPageTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: Implement
        Ok(())
    }
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct PageTableFlags: u64 {
        const PRESENT = 1 << 0;
        const WRITABLE = 1 << 1;
        const USER = 1 << 2;
        const WRITE_THROUGH = 1 << 3;
        const NO_CACHE = 1 << 4;
        const ACCESSED = 1 << 5;
        const DIRTY = 1 << 6;
        const HUGE_PAGE = 1 << 7;
        const GLOBAL = 1 << 8;
        const BIT_9 = 1 << 9;
        const BIT_10 = 1 << 10;
        const BIT_11 = 1 << 11;
        const BIT_52 = 1 << 52;
        const BIT_53 = 1 << 53;
        const BIT_54 = 1 << 54;
        const BIT_55 = 1 << 55;
        const BIT_56 = 1 << 56;
        const BIT_57 = 1 << 57;
        const BIT_58 = 1 << 58;
        const BIT_59 = 1 << 59;
        const BIT_60 = 1 << 60;
        const PKEY_D = 1 << 61;
        const PKEY_U = 1 << 62;
        const NO_EXECUTE = 1 << 63;
    }
}

pub trait Mapper<S: PageSize = Size4KiB> {
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

impl Mapper<Size4KiB> for KernelPageTable {
    unsafe fn map(&mut self, page: Page<Size4KiB>, frame: PhysFrame<Size4KiB>, flags: PageTableFlags) {
        todo!()
    }

    unsafe fn map_with_allocator(
        &mut self,
        page: Page<Size4KiB>,
        frame: PhysFrame<Size4KiB>,
        flags: PageTableFlags,
        frame_alloc: &mut impl FrameAllocator<Size4KiB>,
    ) {
        #[cfg(target_arch = "x86_64")]
        {
            let addr = page.start_address();
            let pt = self.get_or_create_pt(addr.p4_index(), addr.p3_index(), addr.p2_index(), frame_alloc);
            pt[addr.p1_index()].set_frame(frame, flags);
        }
    }

    unsafe fn unmap(&mut self, page: Page<Size4KiB>) {
        todo!()
    }
}

pub struct Flush {
    addr: VirtAddr,
}

#[cfg(target_arch = "x86_64")]
impl Flush {
    pub fn flush(self) {
        // SAFETY: The address is a valid and mapped page
        unsafe { crate::arch::instructions::invlpg(self.addr) };
    }
}
