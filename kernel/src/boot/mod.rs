#[cfg(target_arch = "x86_64")]
pub mod limine;

mod frame_allocator;
mod info;
mod memory_map;
mod page_table;

/// The Main Kernel Entry Function
/// This macro has to be expanded in the main.rs file so that the `kernel_info` symbol is exported
#[macro_export]
macro_rules! kernel_entry {
    () => {
        /// # Safety
        /// This function is unsafe because it can only be called in the initial state
        /// after the bootloader passes control to the kernel
        #[unsafe(export_name = "kernel_entry")]
        unsafe extern "C" fn kernel_entry() -> ! {
            unsafe { $crate::kernel_entry() };
        }
    };
}
