//! Tests for the initial boot structure.

#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(hadron_test::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![allow(unexpected_cfgs)]

use core::ptr::NonNull;

use hadron_test::test_entry;

test_entry!(kernel_entry);

#[test_case]
#[cfg(any(kernel_bootloader = "limine", feature = "never"))]
fn test_memory_map() {
    use hadron_kernel::boot::arch::memory_map::MemoryMap;
    use limine::memory_map::{MemoryMapEntry, MemoryMapEntryType};
    // We make sure that it doesn't crash if we have too many entries
    const BUF_SIZE: usize = MemoryMap::SIZE as usize + 8;
    let mut buffer = [MemoryMapEntry {
        base: 0,
        length: 0,
        ty: MemoryMapEntryType::Usable,
    }; BUF_SIZE];
    let mut ptrs = [NonNull::dangling(); BUF_SIZE];
    for (i, entry) in buffer.iter_mut().enumerate() {
        ptrs[i] = NonNull::new(entry).unwrap();
    }

    let response =
        limine::response::MemoryMapResponse::internal_new(0, BUF_SIZE as u64, NonNull::new(&raw mut ptrs[0]).unwrap());
    let mut memory_map = MemoryMap::default();
    core::hint::black_box(memory_map.parse_from_limine(&response));
}
