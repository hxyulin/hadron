use alloc::{boxed::Box, vec::Vec};
use core::fmt::Write;
use log::Log;
use no_alloc::ringbuf::RingBuf;
use spin::{Mutex, MutexGuard};

use super::timer::time_since_boot;

pub mod console;
pub mod framebuffer;
pub mod serial;

pub enum WriterType {
    Serial,
    Framebuffer,
}

pub trait Writer: Write + Send + Sync {
    fn get_type(&self) -> WriterType;
}

/// The main writer for the kernel, which can be used for logging.
pub struct BufferedWriter {
    buffer: Mutex<RingBuf<u8, 1024>>,
    outputs: Mutex<Vec<Box<dyn Writer>>>,
}

impl BufferedWriter {
    const BUFFER_SIZE: usize = 1024;

    pub const fn empty() -> Self {
        Self {
            buffer: Mutex::new(RingBuf::new()),
            outputs: Mutex::new(Vec::new()),
        }
    }

    pub fn outputs(&self) -> MutexGuard<'_, Vec<Box<dyn Writer>>> {
        self.outputs.lock()
    }

    pub fn add_output(&self, output: Box<dyn Writer>) {
        self.outputs.lock().push(output);
    }

    fn buf_flush(&self, buf: &mut RingBuf<u8, 1024>) {
        let mut outputs = self.outputs.lock();
        while let Some(c) = buf.pop() {
            for device in outputs.iter_mut() {
                device.write_char(c as char).unwrap();
            }
        }
    }

    pub fn flush(&self) {
        let mut buffer = self.buffer.lock();
        self.buf_flush(&mut buffer);
    }

    // PERF: This is very slow, and flushing takes over 0.06s
    pub fn write_str(&self, s: &str) -> core::fmt::Result {
        let mut buffer = self.buffer.lock();
        if buffer.len() + s.len() < Self::BUFFER_SIZE {
            for c in s.as_bytes() {
                // SAFETY: We checked earlier it doesn't overflow
                unsafe { buffer.push_unchecked(*c) };
            }
        } else {
            self.buf_flush(&mut buffer);
            for c in s.as_bytes() {
                // SAFETY: We checked earlier it doesn't overflow
                unsafe { buffer.push_unchecked(*c) };
            }
        }
        Ok(())
    }

    pub fn write_fmt(&self, args: core::fmt::Arguments) -> core::fmt::Result {
        struct Writer<'a> {
            inner: &'a BufferedWriter,
        }

        impl core::fmt::Write for Writer<'_> {
            fn write_str(&mut self, s: &str) -> core::fmt::Result {
                self.inner.write_str(s)
            }
        }

        Writer { inner: self }.write_fmt(args)
    }
}

pub struct KernelLogger;

impl Log for KernelLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        let _ = WRITER.write_fmt(format_args!(
            "[{:.5}] {} {}\n",
            time_since_boot().as_secs_f64(),
            record.level(),
            record.args()
        ));
    }

    fn flush(&self) {
        WRITER.flush();
    }
}

/// The default writer for the kernel, which can be used for logging kernel messages.
/// At early boot, this will print to serial and/or the framebuffer.
pub static WRITER: BufferedWriter = BufferedWriter::empty();

/// The logger for the kernel, which can be used for logging kernel messages.
/// This uses [`WRITER`] internally.
pub static LOGGER: KernelLogger = KernelLogger;

/// Prints a formatted string to the default writer.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => { $crate::util::logging::WRITER.write_fmt(format_args!($($arg)*)).unwrap() }
}

/// Prints a formatted string to the default writer, appending a newline.
#[macro_export]
macro_rules! println {
    // Empty string, just print a newline
    () => { $crate::print!("\n") };
    // No arguments, just print the string
    (fmt:expr) => { $crate::print!(concat!($fmt, "\n")) };
    // One or more arguments, print the string and the arguments
    ($fmt:expr, $($arg:tt)*) => { $crate::print!(concat!($fmt, "\n"), $($arg)*) };
}
