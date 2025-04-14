use alloc::{boxed::Box, vec::Vec};
use core::fmt::Write;
use log::Log;
use spin::{Mutex, MutexGuard};

use super::timer::time_since_boot;

pub mod framebuffer;
pub mod serial;

pub enum WriterType {
    Serial,
    Framebuffer,
}

pub trait Writer: Write + Send + Sync {
    fn get_type(&self) -> WriterType;
}

struct FallbackFnTable {
    write_str: fn(&str) -> core::fmt::Result,
    write_fmt: fn(core::fmt::Arguments<'_>) -> core::fmt::Result,
}

impl FallbackFnTable {
    const fn default() -> Self {
        Self {
            write_str: fallback_write_str,
            write_fmt: fallback_write_fmt,
        }
    }
}

fn fallback_write_str(_s: &str) -> core::fmt::Result {
    Ok(())
}
fn fallback_write_fmt(_args: core::fmt::Arguments<'_>) -> core::fmt::Result {
    Ok(())
}

/// The main writer for the kernel, which can be used for logging.
pub struct KernelWriter {
    outputs: Mutex<Vec<Box<dyn Writer>>>,
    fallback: Mutex<FallbackFnTable>,
}

impl KernelWriter {
    pub const fn empty() -> Self {
        Self {
            outputs: Mutex::new(Vec::new()),
            fallback: Mutex::new(FallbackFnTable::default()),
        }
    }

    pub fn outputs(&self) -> MutexGuard<'_, Vec<Box<dyn Writer>>> {
        self.outputs.lock()
    }

    pub fn add_output(&self, output: Box<dyn Writer>) {
        self.outputs.lock().push(output);
    }

    pub fn remove_fallback(&self) {
        let mut fallback = self.fallback.lock();
        fallback.write_str = fallback_write_str;
        fallback.write_fmt = fallback_write_fmt;
    }

    pub fn add_fallback(
        &self,
        write_str: fn(&str) -> core::fmt::Result,
        write_fmt: fn(core::fmt::Arguments<'_>) -> core::fmt::Result,
    ) {
        let mut fallback = self.fallback.lock();
        fallback.write_str = write_str;
        fallback.write_fmt = write_fmt;
    }

    pub fn write_str(&self, s: &str) -> core::fmt::Result {
        let mut outputs = self.outputs.lock();
        if outputs.is_empty() {
            (self.fallback.lock().write_str)(s)?;
        } else {
            for device in outputs.iter_mut() {
                device.write_str(s)?;
            }
        }
        Ok(())
    }

    pub fn write_fmt(&self, args: core::fmt::Arguments<'_>) -> core::fmt::Result {
        let mut outputs = self.outputs.lock();
        if outputs.is_empty() {
            (self.fallback.lock().write_fmt)(args)?;
        } else {
            for device in outputs.iter_mut() {
                device.write_fmt(args)?;
            }
        }
        Ok(())
    }
}

pub struct KernelLogger;

impl Log for KernelLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        let _ = WRITER.write_fmt(format_args!(
            "[{:.5}] {}: {}\n",
            time_since_boot().as_secs_f64(),
            record.level(),
            record.args()
        ));
    }

    fn flush(&self) {}
}

pub static LOGGER: KernelLogger = KernelLogger;
pub static WRITER: KernelWriter = KernelWriter::empty();

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => { $crate::util::logging::WRITER.write_fmt(format_args!($($arg)*)).unwrap() }
}

#[macro_export]
macro_rules! println {
    () => { $crate::print!("\n") };
    (fmt:expr) => { $crate::print!(concat!($fmt, "\n")) };
    ($fmt:expr, $($arg:tt)*) => { $crate::print!(concat!($fmt, "\n"), $($arg)*) };
}
