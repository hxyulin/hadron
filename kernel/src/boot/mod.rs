#[cfg(target_arch = "x86_64")]
pub mod limine;

#[macro_export]
macro_rules! kernel_entry {
    () => {
        #[unsafe(export_name = "kernel_entry")]
        extern "C" fn kernel_entry() -> ! {
            $crate::kernel_entry();
        }
    };
}
