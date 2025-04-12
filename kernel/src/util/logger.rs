use alloc::{format, vec::Vec};
use core::fmt::Write;
use log::Log;
use spin::RwLock;

use crate::{
    base::info::kernel_info,
    devices::{DeviceId, tty::TtyDevice},
};

use super::timer::time_since_boot;

/// The main logger for the kernel.
/// In initial stages, this prints directly to the serial port devices or framebuffers.
/// Later on, the output will be redirected to a open file as a TTY device, and flushed using
/// another process
pub struct KernelLogger {
    /// The TTY devices to log to
    outputs: RwLock<Vec<DeviceId>>,
}

impl KernelLogger {
    pub const fn empty() -> Self {
        Self {
            outputs: RwLock::new(Vec::new()),
        }
    }

    pub fn add_output(&self, device: DeviceId) {
        self.outputs.write().push(device);
    }
}

impl Log for KernelLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        let devices = &kernel_info().devices;
        let outputs = self.outputs.read();
        for device in outputs.iter() {
            if let Some(device) = devices.get_tty_device(*device) {
                let mut device = device.lock();
                let mut writer = TtyDeviceWriter { tty: &mut *device };
                writer
                    .write_fmt(format_args!(
                        "[{:.5}] {:<5}: {}\n",
                        time_since_boot().as_secs_f32(),
                        record.level(),
                        record.args()
                    ))
                    .unwrap();
            }
        }
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

pub static LOGGER: KernelLogger = KernelLogger::empty();
