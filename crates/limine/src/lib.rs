//! Provides an API for the Limine Boot Protocol.
//! It is based on the [Limine Boot Protocol Specification](https://github.com/limine-bootloader/limine/blob/master/PROTOCOL.md).

#![no_std]

pub mod request;
pub mod response;
pub mod framebuffer;
pub mod memory_map;
pub mod file;
pub mod module;

use core::cell::UnsafeCell;

#[repr(C)]
pub struct BaseRevision {
    magic: [u64; 2],
    // We need to use an UnsafeCell here because it is mutable, so we need to tell the compiler
    // that it can change. (e.g. by Limine writing to it)
    revision: UnsafeCell<u64>,
}

impl BaseRevision {
    /// The magic number of the base revision.
    const MAGIC: [u64; 2] = [0xf9562b2d5c95a6c8, 0x6a7b384944536bdc];

    pub const fn new(revision: u64) -> Self {
        assert!(revision <= 3, "Limine only supports revision up to 3");
        Self {
            magic: Self::MAGIC,
            revision: UnsafeCell::new(revision),
        }
    }

    pub const fn newest() -> Self {
        Self::new(3)
    }

    pub fn is_supported(&self) -> bool {
        let rev = unsafe { self.revision.get().read_volatile() };
        rev == 0
    }
}

unsafe impl Sync for BaseRevision {}
unsafe impl Send for BaseRevision {}

