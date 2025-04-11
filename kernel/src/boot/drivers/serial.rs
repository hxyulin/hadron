use core::fmt::Write;
use spin::Mutex;
use uart_16550::SerialPort;

#[derive(Debug)]
pub struct SerialWriter {
    port: SerialPort,
}

impl SerialWriter {
    pub const fn new(port: u16) -> Self {
        Self {
            port: unsafe { SerialPort::new(port) },
        }
    }

    pub fn init(&mut self) {
        self.port.init();
    }
}

impl Write for SerialWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.port.write_str(s)
    }

    fn write_fmt(&mut self, args: core::fmt::Arguments<'_>) -> core::fmt::Result {
        self.port.write_fmt(args)
    }

    fn write_char(&mut self, c: char) -> core::fmt::Result {
        self.port.write_char(c)
    }
}
