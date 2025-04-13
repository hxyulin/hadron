use alloc::vec::Vec;
use core::fmt::Write;
use log::Log;
use spin::RwLock;

use crate::{
    base::info::kernel_info,
    devices::{DeviceId, tty::TtyDevice},
};

use super::timer::time_since_boot;

/// The main writer for the kernel, which can be used for logging.
/// In initial stages, this prints directly to the serial port devices or framebuffers.
/// Later on, the output will be redirected to a open file as a TTY device, and flushed using
/// another process
pub struct KernelWriter {
    /// The TTY devices to log to
    outputs: RwLock<Vec<DeviceId>>,
}

impl KernelWriter {
    pub const fn empty() -> Self {
        Self {
            outputs: RwLock::new(Vec::new()),
        }
    }

    pub fn add_output(&self, device: DeviceId) {
        self.outputs.write().push(device);
    }

    pub fn write_str(&self, s: &str) -> core::fmt::Result {
        let devices = &kernel_info().devices;
        let outputs = self.outputs.read();
        for device in outputs.iter() {
            if let Some(device) = devices.get_tty_device(*device) {
                let mut device = device.lock();
                let mut writer = TtyDeviceWriter { tty: &mut *device };
                writer.write_str(s)?;
            }
        }
        Ok(())
    }

    pub fn write_fmt(&self, args: core::fmt::Arguments<'_>) -> core::fmt::Result {
        let devices = &kernel_info().devices;
        let outputs = self.outputs.read();
        for device in outputs.iter() {
            if let Some(device) = devices.get_tty_device(*device) {
                let mut device = device.lock();
                let mut writer = TtyDeviceWriter { tty: &mut *device };
                writer.write_fmt(args)?;
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
            "[{}] {}: {}\n",
            time_since_boot().as_secs_f64(),
            record.level(),
            record.args()
        ));
    }

    fn flush(&self) {}
}

struct TtyDeviceWriter<'a> {
    tty: &'a mut dyn TtyDevice,
}

impl<'a> Write for TtyDeviceWriter<'a> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.tty.write(s.as_bytes());
        Ok(())
    }
}

pub static LOGGER: KernelLogger = KernelLogger;
pub static WRITER: KernelWriter = KernelWriter::empty();

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => { _ = $crate::util::logger::WRITER.write_fmt(format_args!($($arg)*)) }
}

#[macro_export]
macro_rules! println {
    () => { $crate::print!("\n") };
    (fmt:expr) => { $crate::print!(concat!($fmt, "\n")) };
    ($fmt:expr, $($arg:tt)*) => { _ = $crate::print!(concat!($fmt, "\n"), $($arg)*) };
}
