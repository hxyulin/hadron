//! Hadron Kernel
//! This contains the code of the core of the kernel
//! Things like builtin drivers, and modules are not included here
//! They are either loaded during runtime, or compiled into the kernel (still loaded at early boot)

#![no_std]
#![no_main]
#![feature(
    abi_x86_interrupt,
    allocator_api,
    vec_push_within_capacity,
    unsafe_cell_access,
    tuple_trait
)]
#![allow(dead_code, clippy::new_without_default)]
// These features are needed for Arc
// We can remove them once they become stable
#![feature(unsize, dispatch_from_dyn, coerce_unsized)]

use base::mem::allocator::KernelAllocator;
use x86_64::PhysAddr;

pub mod base;
pub mod util;

extern crate alloc;

#[global_allocator]
pub static ALLOCATOR: KernelAllocator = KernelAllocator::empty();

#[derive(Debug, Clone, Copy)]
pub struct KernelParams {
    pub rsdp: PhysAddr,
}

