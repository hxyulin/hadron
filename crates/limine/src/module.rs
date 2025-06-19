//! Types for representing modules.
//! This is not entirely implemented yet.

use core::{
    ffi::c_char,
    ops::{BitOr, BitOrAssign},
};

/// Flags of a module
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ModuleFlags(u64);

impl ModuleFlags {
    pub const REQUIRED: u64 = 1 << 0;
    pub const COMPRESSED: u64 = 1 << 1;
    pub const EMPTY: u64 = 0;
}

impl BitOr for ModuleFlags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        ModuleFlags(self.0 | rhs.0)
    }
}

impl BitOrAssign for ModuleFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

#[repr(C)]
pub struct InternalModule {
    name: *const c_char,
    cmdline: *const c_char,
    flags: u64,
}

impl InternalModule {
    pub fn flags(&self) -> ModuleFlags {
        ModuleFlags(self.flags)
    }
}
