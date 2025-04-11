//! Tests for the frame allocator

#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(hadron_test::test_runner)]
#![reexport_test_harness_main = "test_main_real"]
#![allow(unexpected_cfgs)]

use hadron_kernel::ALLOCATOR;
use hadron_test::test_entry;

test_entry!(kernel_entry);

static mut HEAP: [u8; 1024 * 1024] = [0; 1024 * 1024];

extern crate alloc;

/// A bit of a hack to setup the heap
fn test_main() {
    unsafe {
        #[allow(static_mut_refs)]
        ALLOCATOR.init_generic(HEAP.as_mut_ptr(), HEAP.len());
    }

    test_main_real();
}

/// # History
/// This test was added because the same frame was allocated twice to the page table, causing a
/// very hard to debug panic.
#[test_case]
#[cfg(any(kernel_bootloader = "limine", feature = "never"))]
fn test_frame_allocator() {
    use alloc::collections::btree_set::BTreeSet;
    use hadron_kernel::{
        base::mem::{frame_allocator::KernelFrameAllocator, memory_map::MemoryMap},
        boot::arch::memory_map::{BootstrapMemoryMap, MemoryMapEntry, MemoryRegionType},
    };
    use x86_64::{PhysAddr, structures::paging::FrameAllocator};

    let entries = [MemoryMapEntry::new(
        PhysAddr::new(0x1000),
        0x5000,
        MemoryRegionType::Usable,
    )];
    let bootstrap_memory_map = BootstrapMemoryMap::new(&entries);
    let memory_map = MemoryMap::from_bootstrap(&bootstrap_memory_map);
    let mut frame_allocator = KernelFrameAllocator::new(memory_map);

    // We make sure we can actually allocate 5 farmes

    let f1 = frame_allocator.allocate_frame().unwrap();
    let f2 = frame_allocator.allocate_frame().unwrap();
    let f3 = frame_allocator.allocate_frame().unwrap();
    let f4 = frame_allocator.allocate_frame().unwrap();
    let f5 = frame_allocator.allocate_frame().unwrap();
    assert_eq!(frame_allocator.allocate_frame(), None);

    // Make sure we can't allocate the same frame twice

    let mut start_addresses = BTreeSet::new();
    assert!(
        start_addresses.insert(f1.start_address()),
        "Frame 1 was already allocated"
    );
    assert!(
        start_addresses.insert(f2.start_address()),
        "Frame 2 was already allocated"
    );
    assert!(
        start_addresses.insert(f3.start_address()),
        "Frame 3 was already allocated"
    );
    assert!(
        start_addresses.insert(f4.start_address()),
        "Frame 4 was already allocated"
    );
    assert!(
        start_addresses.insert(f5.start_address()),
        "Frame 5 was already allocated"
    );
}
