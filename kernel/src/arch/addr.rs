use core::fmt;

#[derive(Debug, Clone, Copy)]
pub struct InvalidVirtAddr;

impl fmt::Display for InvalidVirtAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid virtual address")
    }
}

impl core::error::Error for InvalidVirtAddr {}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct VirtAddr(usize);

impl fmt::Debug for VirtAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("VirtAddr({:#x})", self.0))
    }
}

#[cfg(target_pointer_width = "64")]
impl VirtAddr {
    /// Checks if an address is canonical
    const fn is_canonical(addr: usize) -> bool {
        if (addr & 0x0000_8000_0000_0000) != 0 {
            (addr & 0xFFFF_0000_0000_0000) == 0xFFFF_0000_0000_0000
        } else {
            (addr & 0xFFFF_0000_0000_0000) == 0
        }
    }

    /// Creates a VirtAddr, and canonicalizing it
    pub const fn new_truncate(addr: usize) -> Self {
        if (addr & 0x0000_8000_0000_0000) != 0 {
            Self((addr & 0x0000_FFFF_FFFF_FFFF) | 0xFFFF_0000_0000_0000)
        } else {
            Self(addr & 0x0000_FFFF_FFFF_FFFF)
        }
    }

    /// # Panics
    /// This function panics if the address is not canonical.
    /// If you want to get an error type instead, see [`VirtAddr::try_new`]
    pub const fn new(addr: usize) -> Self {
        match Self::try_new(addr) {
            Ok(addr) => addr,
            Err(_) => panic!("virtual address is not canonical"),
        }
    }

    /// # Errors
    /// This function will return a InvalidVirtAddr if the address is not canonical
    pub const fn try_new(addr: usize) -> Result<Self, InvalidVirtAddr> {
        if !Self::is_canonical(addr) {
            return Err(InvalidVirtAddr);
        }
        // SAFETY: We checked that it is canonical
        Ok(unsafe { Self::new_unchecked(addr) })
    }

    // # SAFETY
    //
    // This function is unsafe because it does not check if the address is canonical
    pub const unsafe fn new_unchecked(addr: usize) -> Self {
        Self(addr)
    }

    pub const fn as_u64(self) -> u64 {
        // SAFETY: This is only compiled where pointer width is 64
        unsafe { core::mem::transmute(self.0) }
    }
}

impl VirtAddr {
    pub const fn as_usize(self) -> usize {
        self.0
    }

    pub const fn as_ptr<T>(self) -> *const T {
        self.0 as *const T
    }

    pub const fn as_mut_ptr<T>(self) -> *mut T {
        self.0 as *mut T
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PhysAddr(usize);
