use crate::dev::console::ConsoleDevVTable;

pub mod platform;

#[derive(Debug)]
pub struct DriverCapabilities {
    pub console: Option<&'static ConsoleDevVTable>,
}

impl const Default for DriverCapabilities {
    fn default() -> Self {
        Self { console: None }
    }
}
