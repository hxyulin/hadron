use crate::devices::{Device, tty::TtyDevice};

pub struct SerialDevice {
    port: uart_16550::SerialPort,
}

impl SerialDevice {
    pub fn from_initialized_port(port: uart_16550::SerialPort) -> Self {
        Self { port }
    }
}

impl Device for SerialDevice {
    fn as_any(&self) -> &dyn core::any::Any {
        self
    }

    fn class_id(&self) -> crate::devices::DeviceClass {
        crate::devices::DeviceClass::Tty
    }
}

impl TtyDevice for SerialDevice {
    fn capabilities(&self) -> crate::devices::tty::TtyDeviceCapabilities {
        crate::devices::tty::TtyDeviceCapabilities::READ | crate::devices::tty::TtyDeviceCapabilities::WRITE
    }

    fn read(&mut self, buf: &mut [u8]) -> usize {
        let len = buf.len();
        for byte in buf {
            *byte = self.port.receive();
        }
        len
    }

    fn write(&mut self, buf: &[u8]) -> usize {
        let len = buf.len();
        for byte in buf {
            self.port.send_raw(*byte);
        }
        len
    }
}
