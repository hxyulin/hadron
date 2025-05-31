#![no_std]
#![no_main]

mod limine;
pub use limine::kernel_entry;

/// Entrypoint Macro
/// This macro is required as it provides the symbol "kernel_entry", which is the entrypoint to the
/// kernel
///
/// # Example
/// ```no_run
/// entry!()
/// ```
#[macro_export]
macro_rules! entry {
    () => {
        #[unsafe(export_name = "kernel_entry")]
        fn kernel_entry() -> ! {
            $crate::kernel_entry()
        }
    };
}
