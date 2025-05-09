//! Interoperability with C-ABI
//!
//! This module exposes hadron functions that can be called from C-ABI.

use core::fmt::Arguments;

#[unsafe(no_mangle)]
extern "Rust" fn printk(args: Arguments) {
    crate::util::logging::WRITER.write_fmt(args).unwrap();
}
