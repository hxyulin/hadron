#![no_std]
#![no_main]
#![feature(
    allocator_api,
    custom_test_frameworks,
    unsafe_cell_access,
    abi_x86_interrupt,
    const_trait_impl,
    macro_metavar_expr_concat,
)]
#![reexport_test_harness_main = "test_main"]
#![test_runner(crate::tests::test_runner)]

mod boot;
pub use boot::limine::entry as kernel_entry;
pub mod arch;
pub mod mm;
pub mod sync;
pub mod util;

#[unsafe(no_mangle)]
pub extern "Rust" fn kernel_main() -> ! {
    #[cfg(test)]
    test_main();

    loop {}
}

#[cfg(test)]
mod tests {
    use super::*;

    kernel_entry!();

    pub trait Testable {
        fn run(&self);
    }

    impl<T> Testable for T
    where
        T: Fn(),
    {
        fn run(&self) {
            self()
        }
    }

    pub(crate) fn test_runner(tests: &[&dyn Testable]) {
        for test in tests {
            test.run();
        }
    }
}
