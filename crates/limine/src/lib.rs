//! Provides an API for the Limine Boot Protocol.
//! It is based on the [Limine Boot Protocol Specification](https://github.com/limine-bootloader/limine/blob/master/PROTOCOL.md).

#![no_std]

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

#[repr(C)]
pub struct StartMarker {
    magic: [u64; 4],
}

impl StartMarker {
    const MAGIC: [u64; 4] = [
        0xf6b8f4b39de7d1ae,
        0xfab91a6940fcb9cf,
        0x785c6ed015d3e316,
        0x181e920a7852b9d9,
    ];
    pub const fn new() -> Self {
        Self { magic: Self::MAGIC }
    }
}

#[repr(C)]
pub struct EndMarker {
    magic: [u64; 2],
}

impl EndMarker {
    const MAGIC: [u64; 2] = [0xadc0e0531bb10d03, 0x9572709f31764c62];

    pub const fn new() -> Self {
        Self { magic: Self::MAGIC }
    }
}

struct BaseRequest {
    pub id: [u64; 4],
    pub revision: u64,
    pub response_address: u64,
}

impl BaseRequest {
    const MAGIC: [u64; 2] = [0xc7b1dd30df4c8b88, 0x0a82e883a194f07b];
    const BOOTLOADER_INFO: [u64; 4] = concat_arrays::<2, 2, 4>(Self::MAGIC, [0xf55038d8e2a1202f, 0x279426fcf5f59740]);
}

struct BaseResponse {
    pub revision: u64,
}

const fn concat_arrays<const N: usize, const M: usize, const O: usize>(a: [u64; N], b: [u64; M]) -> [u64; O] {
    let mut out = [0u64; O];
    let mut i = 0;
    while i < N {
        out[i] = a[i];
        i += 1;
    }
    let mut j = 0;
    while j < M {
        out[N + j] = b[j];
        j += 1;
    }
    out
}
