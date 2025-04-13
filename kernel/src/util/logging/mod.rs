use alloc::{boxed::Box, vec::Vec};
use core::fmt::Write;
use log::Log;
use spin::Mutex;

use super::timer::time_since_boot;
use crate::base::info::kernel_info;

pub mod framebuffer;
pub mod serial;

/// The main writer for the kernel, which can be used for logging.
/// In initial stages, this prints directly to the serial port devices or framebuffers.
/// Later on, the output will be redirected to a open file as a TTY device, and flushed using
/// another process
pub struct KernelWriter {
    /// The TTY devices to log to
    outputs: Mutex<Vec<Box<dyn Write + Send + Sync>>>,
}

impl KernelWriter {
    pub const fn empty() -> Self {
        Self {
            outputs: Mutex::new(Vec::new()),
        }
    }

    pub fn add_output(&self, output: Box<dyn Write + Send + Sync>) {
        self.outputs.lock().push(output);
    }

    pub fn write_str(&self, s: &str) -> core::fmt::Result {
        let mut outputs = self.outputs.lock();
        for device in outputs.iter_mut() {
            device.write_str(s)?;
        }
        Ok(())
    }

    pub fn write_fmt(&self, args: core::fmt::Arguments<'_>) -> core::fmt::Result {
        let mut outputs = self.outputs.lock();
        for device in outputs.iter_mut() {
            device.write_fmt(args)?;
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
