use core::fmt::Write;

use log::Log;
use no_alloc::ringbuf::RingBuf;
use spin::Mutex;

struct KernelLogger {
    buf: RingBuf<u8, 2048>,
    fallback: Option<FallbackLogger>,
}

impl core::fmt::Write for KernelLogger {
    fn write_char(&mut self, c: char) -> core::fmt::Result {
        if let Err(c) = self.buf.push(c as u8) {
            self.flush();
            self.buf.push(c).expect("Failed to write to RingBuf");
        }
        Ok(())
    }
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.as_bytes() {
            self.write_char(*c as char)?;
        }
        Ok(())
    }
}

impl KernelLogger {
    pub const fn new() -> KernelLogger {
        Self {
            buf: RingBuf::new(),
            fallback: None,
        }
    }

    pub const fn set_fallback(&mut self, fallback: FallbackLogger) -> Option<FallbackLogger> {
        self.fallback.replace(fallback)
    }

    pub fn flush(&mut self) {
        while let Some(c) = self.buf.pop() {
            if let Some(fallback) = self.fallback.as_mut() {
                fallback.write_char(c as char);
            }
        }
    }
}

pub struct FallbackLogger {
    serial: uart_16550::SerialPort,
}

impl FallbackLogger {
    pub fn new() -> FallbackLogger {
        // SAFETY: 0x3F8 is valid serial port (COM1)
        let mut serial = unsafe { uart_16550::SerialPort::new(0x3F8) };
        serial.init();
        Self { serial }
    }
    pub fn write_char(&mut self, c: char) {
        self.serial.write_char(c);
    }
}

pub struct Logger {
    inner: Mutex<KernelLogger>,
}

impl Logger {
    pub fn set_fallback(&self, fallback: FallbackLogger) -> Option<FallbackLogger> {
        self.inner.lock().set_fallback(fallback)
    }
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        let mut logger = self.inner.lock();
        writeln!(logger, "{}", record.args());
    }

    fn flush(&self) {
        self.inner.lock().flush();
    }
}

pub static LOGGER: Logger = Logger {
    inner: Mutex::new(KernelLogger::new()),
};

pub fn init_log() {
    let _ = log::set_logger(&LOGGER);
    log::set_max_level(log::LevelFilter::Trace);
}
