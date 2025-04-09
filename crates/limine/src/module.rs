use core::ffi::c_char;

const MODULE_FLAGS_REQUIRED: u64 = 1 << 0;
const MODULE_FLAGS_COMPRESSED: u64 = 1 << 1;

#[repr(C)]
pub struct InternalModule {
    name: *const c_char,
    cmdline: *const c_char,
    flags: u64,
}
