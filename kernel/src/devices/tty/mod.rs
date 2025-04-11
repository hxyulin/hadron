use super::Device;

pub mod fb;
pub mod serial;

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct TtyDeviceCapabilities: u8 {
        const READ = 1 << 0;
        const WRITE = 1 << 1;
    }
}

pub trait TtyDevice: Device {
    fn capabilities(&self) -> TtyDeviceCapabilities;
    fn read(&mut self, _buf: &mut [u8]) -> usize {
        0
    }
    fn write(&mut self, _buf: &[u8]) -> usize {
        0
    }
}
