//! Provides an API for the Limine Boot Protocol.
//! It is based on the [Limine Boot Protocol Specification](https://github.com/limine-bootloader/limine/blob/master/PROTOCOL.md).
//! It implements requests and responses as idiomatic Rust types.
//!
//! See [`crate::request`] and [`crate::response`] for more information.

// This crate is designed to be used in a bare-metal environment.
#![no_std]

pub mod file;
pub mod framebuffer;
pub mod memory_map;
pub mod module;
pub mod request;
pub mod response;

use core::cell::UnsafeCell;

/// The base revision of the protocol.
/// This should be placed in memory and checked before any of the responses are read.
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

    /// Creates a new `BaseRevision` with the given revision.
    pub const fn new(revision: u64) -> Self {
        assert!(revision <= 3, "Limine only supports revision up to 3");
        Self {
            magic: Self::MAGIC,
            revision: UnsafeCell::new(revision),
        }
    }

    /// Creates a new `BaseRevision` with the latest revision (that this crate supports).
    pub const fn newest() -> Self {
        Self::new(3)
    }

    /// Returns `true` if the protocol is supported.
    pub fn is_supported(&self) -> bool {
        self.revision() == 0
    }

    /// Returns the revision of the protocol.
    pub fn revision(&self) -> u64 {
        // Safety: We know only have access to the limine requests when in a single-threaded
        // environment, so we can safely read the revision.
        unsafe { self.revision.get().read_volatile() }
    }
}

// SAFETY: The base revision is only ever accessed in a single-threaded environment.
unsafe impl Sync for BaseRevision {}
// SAFETY: The base revision is only ever accessed in a single-threaded environment.
unsafe impl Send for BaseRevision {}
