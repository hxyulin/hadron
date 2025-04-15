//! Interoperability with C-ABI
//!
//! This module exposes hadorn functions that can be called from C-ABI.

extern "C" fn printk(ptr: *const u8, len: usize) {
    // SAFETY: We are passing a pointer to a slice of bytes
    let slice = unsafe { core::slice::from_raw_parts(ptr, len) };
    let string = core::str::from_utf8(slice).unwrap();
    log::info!("{}", string);
}
