use core::fmt;

use alloc::boxed::Box;
use no_alloc::{ringbuf::RingBuf, vec::ArrayVec};
use spin::Mutex;

pub static LOGGER: Mutex<Logger> = Mutex::new(Logger::empty());

pub struct Logger {
    pub ringbuf: RingBuf<u8, 4096>,
    pub loggers: ArrayVec<Box<dyn LogConsole>, 4>,
}

impl Logger {
    const fn empty() -> Self {
        Self {
            ringbuf: RingBuf::new(),
            loggers: ArrayVec::new(),
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

#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => {
        _ = core::fmt::Write::write_fmt(core::ops::DerefMut::deref_mut(&mut $crate::util::kprint::LOGGER.try_lock().unwrap()), format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! kprintln {
    ($fmt:expr) => {$crate::kprint!(concat!($fmt, "\n"))};
    ($fmt:expr, $($arg:tt)*) => {$crate::kprint!(concat!($fmt, "\n"), $($arg)*)};
}
