use core::fmt;

use alloc::{boxed::Box, sync::Arc, vec::Vec};
use no_alloc::ringbuf::RingBuf;
use spin::Mutex;

use crate::dev::Device;

pub static LOGGER: Mutex<Logger> = Mutex::new(Logger::empty());

pub struct Logger {
    pub ringbuf: RingBuf<u8, 4096>,
    pub loggers: Vec<Box<dyn LogConsole>>,
}

impl Logger {
    const fn empty() -> Self {
        Self {
            ringbuf: RingBuf::new(),
            loggers: Vec::new(),
        }
    }
}

impl fmt::Write for Logger {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for logger in self.loggers.iter_mut() {
            for b in s.as_bytes() {
                logger.write_byte(*b);
            }
        }
        Ok(())
    }
}

pub trait LogConsole: Send + Sync {
    fn write_byte(&mut self, byte: u8);
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Debug = 0,
    Info = 1,
    Warn = 2,
    Error = 3,
    Fatal = 4,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Debug => "DEBUG",
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERROR",
            Self::Fatal => "FATAL",
        })
    }
}

pub fn kprint_internal(args: fmt::Arguments) {
    use fmt::Write;
    _ = LOGGER.lock().write_fmt(args);
}

#[macro_export]
macro_rules! kprint {
    ($level:ident, $fmt:expr) => {
        $crate::util::kprint::kprint_internal(format_args!(
            concat!("{} ", $fmt),
            $crate::util::kprint::LogLevel::$level,
        ))
    };
    ($level:ident, $fmt:expr, $($arg:tt)*) => {
        $crate::util::kprint::kprint_internal(format_args!(
            concat!("{} ", $fmt),
            $crate::util::kprint::LogLevel::$level,
            $($arg)*,
        ))
    };
}

#[macro_export]
macro_rules! kprintln {
    ($level:ident, $fmt:literal) => {
        $crate::kprint!($level, concat!($fmt, "\n"))
    };
    ($level:ident, $fmt:literal, $($arg:tt)*) => {
        $crate::kprint!($level, concat!($fmt, "\n"), $($arg)*)
    };
}

pub struct ConsoleWriter {
    device: Arc<Device>,
    write_fn: fn(&Device, u8),
}

impl ConsoleWriter {
    pub fn new(device: &Arc<Device>) -> Self {
        assert!(device.drv.is_some(), "console writer needs a device with a driver");
        assert!(
            device.drv.as_ref().unwrap().caps.console.is_some(),
            "console writer needs a device with the console capability"
        );

        Self {
            device: device.clone(),
            write_fn: device.drv.as_ref().unwrap().caps.console.unwrap().write,
        }
    }
}

impl LogConsole for ConsoleWriter {
    fn write_byte(&mut self, byte: u8) {
        (self.write_fn)(&self.device, byte);
    }
}
