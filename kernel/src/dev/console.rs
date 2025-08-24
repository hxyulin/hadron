use crate::dev::Device;

pub struct ConsoleDevVTable {
    pub write: fn(dev: &Device, char: u8),
}

impl core::fmt::Debug for ConsoleDevVTable {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ConsoleDevVTable")
            .field("write", &format_args!("{:#x}", self.write as usize))
            .finish()
    }
}
