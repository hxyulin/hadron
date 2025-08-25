#![no_std]
#![cfg_attr(not(feature = "test"), no_main)]
#![feature(
    allocator_api,
    custom_test_frameworks,
    unsafe_cell_access,
    abi_x86_interrupt,
    const_trait_impl,
    macro_metavar_expr_concat,
    const_default,
    vec_push_within_capacity
)]
#![cfg_attr(not(feature = "test"), reexport_test_harness_main = "test_main")]
#![cfg_attr(not(feature = "test"), test_runner(crate::tests::test_runner))]

extern crate alloc;

mod boot;

#[cfg(target_arch = "x86_64")]
pub use boot::limine::entry as kernel_entry;
#[cfg(not(target_arch = "x86_64"))]
pub fn kernel_entry() -> ! {
    loop {}
}

#[cfg(feature = "test")]
extern crate std;

pub mod arch;
pub mod dev;
pub mod mm;
pub mod sync;
pub mod util;

#[unsafe(no_mangle)]
pub extern "Rust" fn kernel_main() -> ! {
    #[cfg(all(test, not(feature = "test")))]
    {
        test_main();
        hadron_test::exit_qemu(hadron_test::ExitCode::Success);
    }
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
