#![allow(unexpected_cfgs)]

use core::{panic::PanicInfo, sync::atomic::AtomicBool};

static IS_BOOT: AtomicBool = AtomicBool::new(true);

pub mod limine;

pub mod arch;
pub mod ctor;
pub mod info;

pub fn boot_panic(info: &PanicInfo) -> ! {
    limine::limine_print_panic(info);
    loop {
        x86_64::instructions::hlt();
    }
}

pub fn is_boot() -> bool {
    IS_BOOT.load(core::sync::atomic::Ordering::Relaxed)
}
