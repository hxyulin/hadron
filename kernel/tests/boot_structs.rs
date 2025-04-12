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

/// # History
/// This test was added because the kernel panicked when running on real hardware with 48 GB of RAM.
#[test_case]
#[cfg(any(kernel_bootloader = "limine", feature = "never"))]
fn test_memory_map() {
}
