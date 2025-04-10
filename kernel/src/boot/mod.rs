#![allow(unexpected_cfgs)]

/// We use any here so that IDE services don't break, but the feature `never` is actually never used

#[cfg(any(kernel_bootloader = "limine", feature = "never"))]
pub mod limine;

pub mod arch;
pub mod drivers;
pub mod info;
