#![no_std]

pub mod paging;

/// A canonical virtual address.
/// This means that bits 48-63 are bit extensions of bit 47.
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtAddr(u64);

impl VirtAddr {
    pub const fn new(addr: u64) -> Self {
        match Self::try_new(addr) {
            Some(addr) => addr,
            None => panic!("VirtAddr::new: address is not canonicalized"),
        }
    }

    /// Tries to create a new `VirtAddr` from a raw address.
    pub const fn try_new(addr: u64) -> Option<Self> {
        if Self::is_canonical(addr) {
            Some(Self(addr))
        } else {
            None
        }
    }

    /// Creates a new `VirtAddr` from a raw address.
    /// # Safety
    /// The address must be canonicalized.
    pub const unsafe fn new_unchecked(addr: u64) -> Self {
        Self(addr)
    }

    /// Creates a new `VirtAddr` from a raw address, truncating it and canonicalizing it.
    pub const fn new_truncate(addr: u64) -> Self {
        let mask = ((addr << 16) as i64 >> 15) as u64;
        Self(addr & 0x0000_FFFF_FFFF_FFFF | mask)
    }

    pub const fn canonicalize(self) -> Self {
        let mask = ((self.0 << 16) as i64 >> 15) as u64;
        Self(self.0 & 0x0000_FFFF_FFFF_FFFF | mask)
    }

    const fn is_canonical(addr: u64) -> bool {
        let mask = ((addr << 16) as i64 >> 15) as u64 >> 48;
        (addr >> 48) == mask
    }

    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

impl core::fmt::Debug for VirtAddr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("VirtAddr").field(&format_args!("{:#018X}", self.0)).finish()
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysAddr(u64);

impl PhysAddr {
    pub const fn new(addr: u64) -> Self {
        Self(addr)
    }

    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

impl core::fmt::Debug for PhysAddr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("PhysAddr").field(&format_args!("{:#018X}", self.0)).finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static_assertions::assert_cfg!(target_pointer_width = "64");

    static_assertions::assert_impl_all!(VirtAddr: core::fmt::Debug);
    static_assertions::const_assert_eq!(VirtAddr::new(0).0, 0);
    static_assertions::const_assert!(VirtAddr::is_canonical(0xFFFF_8000_0000_0000));
    static_assertions::const_assert!(VirtAddr::is_canonical(0xFFFF_FFFF_FFFF_FFFF));
    static_assertions::const_assert!(!VirtAddr::is_canonical(0xFFFF_4000_0000_0000));
}
