#[cfg(target_arch = "x86_64")]
mod x86_64;
#[cfg(target_arch = "x86_64")]
pub use x86_64::*;

use crate::{PhysAddr, VirtAddr};

pub trait PageSize {
    const SIZE: usize;
}

pub struct Page<S: PageSize> {
    addr: VirtAddr,
    _marker: core::marker::PhantomData<S>,
}

impl<S: PageSize> Page<S> {
    /// Starts a new page at the given address.
    ///
    /// # Panics
    ///
    /// Panics if the given address is not page-aligned.
    pub const fn start_from(addr: VirtAddr) -> Self {
        assert!(addr.as_u64() & (S::SIZE as u64 - 1) == 0);

        // SAFETY: The address is page-aligned.
        unsafe { Self::start_from_unchecked(addr) }
    }

    /// Starts a new page at the given address.
    ///
    /// # Safety
    ///
    /// The given address must be page-aligned.
    pub const unsafe fn start_from_unchecked(addr: VirtAddr) -> Self {
        Self {
            addr,
            _marker: core::marker::PhantomData,
        }
    }

    /// Returns a page containing the given address.
    pub const fn containing_addr(addr: VirtAddr) -> Self {
        let addr = addr.as_u64() & !(S::SIZE as u64 - 1);
        // SAFETY: The address is page-aligned.
        unsafe { Self::start_from_unchecked(VirtAddr::new(addr)) }
    }

    pub const fn start_addr(self) -> VirtAddr {
        self.addr
    }

    pub const fn size() -> usize {
        S::SIZE
    }
}

pub struct PhysFrame<S: PageSize> {
    addr: PhysAddr,
    _marker: core::marker::PhantomData<S>,
}

impl<S: PageSize> PhysFrame<S> {
    /// Starts a new page at the given address.
    ///
    /// # Panics
    ///
    /// Panics if the given address is not page-aligned.
    pub const fn start_from(addr: PhysAddr) -> Self {
        assert!(addr.as_u64() & (S::SIZE as u64 - 1) == 0);

        // SAFETY: The address is page-aligned.
        unsafe { Self::start_from_unchecked(addr) }
    }

    /// Starts a new page at the given address.
    ///
    /// # Safety
    ///
    /// The given address must be page-aligned.
    pub const unsafe fn start_from_unchecked(addr: PhysAddr) -> Self {
        Self {
            addr,
            _marker: core::marker::PhantomData,
        }
    }

    #[cfg(target_arch = "x86_64")]
    pub const fn start_from_vaddr(addr: VirtAddr) -> Self {
        let addr = addr.as_u64() & !(S::SIZE as u64 - 1);
        // SAFETY: The address is page-aligned.
        unsafe { Self::start_from_unchecked(PhysAddr::new(addr)) }
    }

    pub const fn start_addr(self) -> PhysAddr {
        self.addr
    }

    pub const fn size() -> usize {
        S::SIZE
    }
}
