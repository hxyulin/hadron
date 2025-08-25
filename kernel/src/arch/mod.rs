#[cfg(target_arch = "x86_64")]
pub mod x86_64;

mod addr;
pub use addr::{PhysAddr, VirtAddr};

pub mod instructions;
pub mod registers;
