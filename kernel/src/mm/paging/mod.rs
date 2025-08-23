use core::{fmt, marker::PhantomData};

use crate::arch::{PhysAddr, VirtAddr};

pub trait PageSize: fmt::Debug + Clone + Copy {
    const SIZE: usize;
}

#[derive(Debug, Clone, Copy)]
pub struct Size4KiB;

impl PageSize for Size4KiB {
    const SIZE: usize = 4096;
}

#[derive(Debug, Clone, Copy)]
pub struct Size2MiB;

impl PageSize for Size2MiB {
    const SIZE: usize = 4096 * 512;
}

#[derive(Debug, Clone, Copy)]
pub struct Size1GiB;

impl PageSize for Size1GiB {
    const SIZE: usize = 4096 * 512 * 512;
}

pub struct Page<S: PageSize = Size4KiB> {
    base: VirtAddr,
    _marker: PhantomData<S>,
}

#[derive(Debug, Clone, Copy)]
pub struct AlignmentError;

impl fmt::Display for AlignmentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("alignment error")
    }
}

impl core::error::Error for AlignmentError {}

impl<S: PageSize> Page<S> {
    /// Creates a new page from a start address
    ///
    /// # Panics
    /// This function will panic if the start address is not aligned
    pub const fn from_start_address(base: VirtAddr) -> Self {
        match Self::try_from_start_address(base) {
            Ok(page) => page,
            Err(_) => panic!("start address not aligned"),
        }
    }

    /// Creates a new Page from a start address
    ///
    /// # Errors
    /// This function will error if the start address is not aligned
    pub const fn try_from_start_address(base: VirtAddr) -> Result<Self, AlignmentError> {
        match base.is_aligned(S::SIZE) {
            // SAFETY: We checked that it is aligned to the page size
            true => Ok(unsafe { Self::from_start_address_unchecked(base) }),
            false => Err(AlignmentError),
        }
    }

    /// Creates a new Page from a start address
    ///
    /// # Safety
    /// This function is unsafe because it doesn't check if the start address is aligned
    pub const unsafe fn from_start_address_unchecked(base: VirtAddr) -> Self {
        Self {
            base,
            _marker: PhantomData,
        }
    }

    /// Creates a new Page containing the current address
    pub const fn containing_address(addr: VirtAddr) -> Self {
        let base = VirtAddr::new(addr.as_usize() / S::SIZE * S::SIZE);
        Self::from_start_address(base)
    }

    pub const fn start_address(&self) -> VirtAddr {
        self.base
    }

    pub const fn size(&self) -> usize {
        S::SIZE
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PhysFrame<S: PageSize = Size4KiB> {
    base: PhysAddr,
    _marker: PhantomData<S>,
}

impl<S: PageSize> PhysFrame<S> {
    pub const fn start_address(&self) -> PhysAddr {
        self.base
    }

    pub const fn from_start_address(base: PhysAddr) -> Self {
        assert!(base.is_aligned(S::SIZE), "frame start address is not aligned");
        Self {
            base,
            _marker: PhantomData,
        }
    }
}

pub trait FrameAllocator<S: PageSize> {
    fn allocate_frame(&mut self) -> Option<PhysFrame<S>>;
}

pub trait FrameDeallocator<S: PageSize> {
    fn deallocate_frame(&mut self, frame: PhysFrame);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn page_from_start_addr() {
        let aligned = Page::<Size4KiB>::try_from_start_address(VirtAddr::new(0x0000));
        assert!(aligned.is_ok());
        let unaligned = Page::<Size4KiB>::try_from_start_address(VirtAddr::new(0x1001));
        assert!(unaligned.is_err());
    }

    #[test_case]
    fn page_containing() {
        let page1 = Page::<Size4KiB>::containing_address(VirtAddr::new(0x0000));
        assert_eq!(page1.start_address(), VirtAddr::new(0x0000));

        let page2 = Page::<Size4KiB>::containing_address(VirtAddr::new(0x1001));
        assert_eq!(page2.start_address(), VirtAddr::new(0x1000));

        let page3 = Page::<Size4KiB>::containing_address(VirtAddr::new(0x1FFF));
        assert_eq!(page3.start_address(), VirtAddr::new(0x1000));

        let page4 = Page::<Size4KiB>::containing_address(VirtAddr::new(0x3000));
        assert_eq!(page4.start_address(), VirtAddr::new(0x3000));
    }
}
