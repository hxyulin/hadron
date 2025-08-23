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
    pub const NULL: Self = Self(0);

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

    pub fn from_ptr<T>(ptr: *const T) -> Self {
        Self::new(ptr as usize)
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

    pub fn p4_index(&self) -> usize {
        (self.as_usize() >> 39) & 0x1FF
    }

    pub fn p3_index(&self) -> usize {
        (self.as_usize() >> 30) & 0x1FF
    }

    pub fn p2_index(&self) -> usize {
        (self.as_usize() >> 21) & 0x1FF
    }

    pub fn p1_index(&self) -> usize {
        (self.as_usize() >> 12) & 0x1FF
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

    /// Checks whether the VirtAddr is aligned
    pub const fn is_aligned(&self, alignment: usize) -> bool {
        self.as_usize() % alignment == 0
    }
}

impl core::ops::Add<usize> for VirtAddr {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Self::new_truncate(self.0 + rhs)
    }
}

impl core::ops::AddAssign<usize> for VirtAddr {
    fn add_assign(&mut self, rhs: usize) {
        *self = *self + rhs;
    }
}

#[cfg(target_pointer_width = "64")]
impl core::ops::Add<u64> for VirtAddr {
    type Output = Self;

    fn add(self, rhs: u64) -> Self::Output {
        Self::new(self.0 + rhs as usize)
    }
}

#[cfg(target_pointer_width = "64")]
impl core::ops::AddAssign<u64> for VirtAddr {
    fn add_assign(&mut self, rhs: u64) {
        *self = *self + rhs;
    }
}

impl core::ops::Sub<usize> for VirtAddr {
    type Output = Self;

    fn sub(self, rhs: usize) -> Self::Output {
        Self::new_truncate(self.0 - rhs)
    }
}

impl core::ops::SubAssign<usize> for VirtAddr {
    fn sub_assign(&mut self, rhs: usize) {
        *self = *self - rhs;
    }
}

#[cfg(target_pointer_width = "64")]
impl core::ops::Sub<u64> for VirtAddr {
    type Output = Self;

    fn sub(self, rhs: u64) -> Self::Output {
        Self::new(self.0 - rhs as usize)
    }
}

#[cfg(target_pointer_width = "64")]
impl core::ops::SubAssign<u64> for VirtAddr {
    fn sub_assign(&mut self, rhs: u64) {
        *self = *self - rhs;
    }
}

impl fmt::LowerHex for VirtAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(&self.0, f)
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PhysAddr(usize);

#[cfg(target_pointer_width = "64")]
impl PhysAddr {
    pub const NULL: Self = Self(0);

    pub const fn new(addr: usize) -> Self {
        Self(addr)
    }

    pub const fn as_u64(self) -> u64 {
        self.0 as u64
    }
}

impl PhysAddr {
    pub const fn as_usize(self) -> usize {
        self.0
    }

    pub const fn is_aligned(&self, alignment: usize) -> bool {
        self.as_usize() % alignment == 0
    }
}

impl fmt::Debug for PhysAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("PhysAddr({:#x})", self.0))
    }
}

impl core::ops::Add<usize> for PhysAddr {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Self::new(self.0 + rhs)
    }
}

impl core::ops::AddAssign<usize> for PhysAddr {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs;
    }
}

#[cfg(target_pointer_width = "64")]
impl core::ops::Add<u64> for PhysAddr {
    type Output = Self;

    fn add(self, rhs: u64) -> Self::Output {
        Self::new(self.0 + rhs as usize)
    }
}

impl fmt::LowerHex for PhysAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(&self.0, f)
    }
}
