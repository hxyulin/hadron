use x86_64::{
    PhysAddr, VirtAddr,
    structures::paging::{PageSize, PageTableFlags, PhysFrame, Size4KiB},
};

use crate::base::{info::kernel_info, mem::{map_page, mappings}};

pub struct KernelMmio {
    offset: u64,
}

impl Default for KernelMmio {
    fn default() -> Self {
        Self::new()
    }
}

impl KernelMmio {
    pub const fn new() -> Self {
        Self { offset: 0 }
    }

    pub fn allocate_persistant(&mut self, phys_addr: PhysAddr, size: u64) -> VirtAddr {
        let addr = mappings::MMIO_SPACE + self.offset;
        // Round up to the next page
        self.offset += (size + 0xFFF) & !0xFFF;
        // >> 12 = / 4096
        let pages = (size + 0xFFF) >> 12;

        // It needs to be a certain frame, so we can't use on demand allocation
        for i in 0..pages {
            let offset = i * Size4KiB::SIZE;
            unsafe {
                map_page(
                    PhysFrame::from_start_address(phys_addr + offset).unwrap(),
                    addr + offset,
                    PageTableFlags::PRESENT
                        | PageTableFlags::WRITABLE
                        | PageTableFlags::NO_EXECUTE
                        | PageTableFlags::NO_CACHE,
                );
            }
        }

        addr
    }
}

/// Zero-sized type used to offset an MMIO register.
/// This is aimed to be as a utility wrapper around a MMIO register, so that it can be used as a
/// field in a struct without increasing the size of the struct.
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct OffsetMmio<T, const OFFSET: u64> {
    _marker: core::marker::PhantomData<T>,
}

impl<T, const OFFSET: u64> Default for OffsetMmio<T, OFFSET> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const OFFSET: u64> OffsetMmio<T, OFFSET> {
    pub const fn new() -> Self {
        assert!(OFFSET % core::mem::size_of::<T>() as u64 == 0, "Offset must be aligned");
        Self {
            _marker: core::marker::PhantomData,
        }
    }

    pub fn addr(&self, base: VirtAddr) -> VirtAddr {
        base + OFFSET
    }

    pub fn read(&self, base: VirtAddr) -> T {
        unsafe { core::ptr::read_volatile(self.addr(base).as_mut_ptr::<T>()) }
    }

    pub fn write(&self, base: VirtAddr, value: T) {
        unsafe { core::ptr::write_volatile(self.addr(base).as_mut_ptr::<T>(), value) }
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct DynamicOffsetMmio<T> {
    offset: u64,
    _marker: core::marker::PhantomData<T>,
}

impl<T> DynamicOffsetMmio<T> {
    pub const fn new(offset: u64) -> Self {
        Self {
            offset,
            _marker: core::marker::PhantomData,
        }
    }
    pub fn addr(&self, base: VirtAddr) -> VirtAddr {
        base + self.offset
    }

    pub fn read(&self, base: VirtAddr) -> T {
        unsafe { core::ptr::read_volatile(self.addr(base).as_mut_ptr::<T>()) }
    }

    pub fn write(&self, base: VirtAddr, value: T) {
        unsafe { core::ptr::write_volatile(self.addr(base).as_mut_ptr::<T>(), value) }
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct OffsetMmioArray<T, const OFFSET: u64, const LEN: usize> {
    _marker: core::marker::PhantomData<T>,
}

impl<T, const OFFSET: u64, const LEN: usize> Default for OffsetMmioArray<T, OFFSET, LEN> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const OFFSET: u64, const LEN: usize> OffsetMmioArray<T, OFFSET, LEN> {
    pub const fn new() -> Self {
        assert!(OFFSET % core::mem::size_of::<T>() as u64 == 0, "Offset must be aligned");
        Self {
            _marker: core::marker::PhantomData,
        }
    }

    pub fn get(&self, index: usize) -> DynamicOffsetMmio<T> {
        assert!(index < LEN);
        DynamicOffsetMmio::new(OFFSET + index as u64 * core::mem::size_of::<T>() as u64)
    }
}

pub fn allocate_persistent_mmio(phys_addr: PhysAddr, size: u64) -> VirtAddr {
    let mut mmio = kernel_info().mmio.lock();
    mmio.allocate_persistant(phys_addr, size)
}
