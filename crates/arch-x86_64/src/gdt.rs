use core::ops::{Index, IndexMut};

use kernel_base::VirtAddr;

/// An 80-bit pointer to the GDT.
#[repr(C)]
pub struct GlobalDescriptorTableRegister {
    size: u16,
    /// The virtual address of the GDT.
    ptr: VirtAddr,
}

#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct GlobalDescriptorTable {
    entries: [GlobalDescriptorTableEntry; 8],
}

impl Index<usize> for GlobalDescriptorTable {
    type Output = GlobalDescriptorTableEntry;

    fn index(&self, index: usize) -> &Self::Output {
        &self.entries[index]
    }
}

impl IndexMut<usize> for GlobalDescriptorTable {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.entries[index]
    }
}

impl GlobalDescriptorTable {
    pub fn new() -> Self {
        Self {
            entries: [GlobalDescriptorTableEntry::null(); 8],
        }
    }

    pub unsafe fn load(&self) {
        let gdtr = GlobalDescriptorTableRegister {
            size: core::mem::size_of::<Self>() as u16 - 1,
            ptr: VirtAddr::new(self as *const Self as u64),
        };
        unsafe {
            core::arch::asm!(
                "cli",
                "lgdt [{}]",
                in(reg) &raw const gdtr,
                options(nostack, preserves_flags)
            )
        }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct GlobalDescriptorTableEntry(u64);

impl core::fmt::Debug for GlobalDescriptorTableEntry {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("GlobalDescriptorTableEntry")
            .field("access", &self.access())
            .field("flags", &self.flags())
            .field("limit", &self.limit())
            .field("base", &self.base())
            .finish()
    }
}

impl GlobalDescriptorTableEntry {
    /// Create a null GDT entry.
    pub const fn null() -> Self {
        Self(0)
    }

    pub const fn new(access: AccessByte, flags: GdtFlags, limit: u32, base: u32) -> Self {
        let mut entry = Self::null();
        entry.set_access(access);
        entry.set_flags(flags);
        entry.set_limit(limit);
        entry.set_base(base);
        entry
    }

    pub const fn access(self) -> AccessByte {
        AccessByte::from_bits_truncate(((self.0 >> 40) & 0xFF) as u8)
    }

    pub const fn set_access(&mut self, access: AccessByte) {
        self.0 &= !(0xFF << 40);
        self.0 |= (access.bits() as u64) << 40;
    }

    pub const fn flags(self) -> GdtFlags {
        GdtFlags::from_bits_truncate(((self.0 >> 52) & 0xFF) as u8)
    }

    pub const fn set_flags(&mut self, flags: GdtFlags) {
        self.0 &= !(0xFF << 52);
        self.0 |= (flags.bits() as u64) << 52;
    }

    pub const fn limit(self) -> u32 {
        ((self.0 & 0xFFFF) | ((self.0 >> 48) & 0xF)) as u32
    }

    pub const fn set_limit(&mut self, limit: u32) {
        self.0 &= !(0xFFFF | 0xF << 48);
        self.0 |= (limit as u64 & 0xFFFF) | ((limit as u64 >> 16) & 0xF) << 48;
    }

    pub const fn base(self) -> u32 {
        (((self.0 >> 16) & 0x00FFFFFF) | ((self.0 >> 56) & 0xFF)) as u32
    }

    pub const fn set_base(&mut self, base: u32) {
        self.0 &= !(0x00FFFFFF | 0xFF << 56);
        self.0 |= (base as u64 & 0x00FFFFFF) | ((base as u64 >> 24) & 0xFF) << 56;
    }
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct AccessByte: u8 {
        const PRESENT = 1 << 7;
        const DESC_TYPE = 1 << 4;
        const EXECUTABLE = 1 << 3;
        const DIRECTION = 1 << 2;
        const READ_WRITE = 1 << 1;
        const ACCESSED = 1 << 0;

        const _ = !0;
    }

    #[derive(Debug, Clone, Copy)]
    pub struct GdtFlags: u8 {
        const GRANULARITY = 1 << 3;
        const DB = 1 << 2;
        const LONG_MODE = 1 << 1;
    }
}

impl AccessByte {
    pub const fn privilege_level(self) -> u8 {
        (self.bits() >> 5) & 0b11
    }

    pub fn set_privilege_level(self, level: u8) -> Self {
        assert!(level <= 3);
        Self::from_bits_truncate(self.bits() & !(0b11 << 5) | level << 5)
    }
}
