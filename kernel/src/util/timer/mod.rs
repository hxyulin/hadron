pub mod rtc;
pub mod hpet;

pub use core::time::Duration;

use rtc::UtcTime;

use crate::base::info::try_kernel_info;

pub struct TimerTicks(pub u64);

impl TimerTicks {
    pub const fn new(ticks: u64) -> Self {
        Self(ticks)
    }
}

pub fn time_since_boot() -> Duration {
    if let Some(kernel_info) = try_kernel_info() {
        kernel_info
            .timer
            .get()
            .map(|t| t.read().time_since_boot())
            .unwrap_or_default()
    } else {
        Duration::default()
    }
}

pub fn current_time() -> UtcTime {
    rtc::read_time()
}

pub trait Timer {
    fn now(&self) -> TimerTicks;
    fn frequency(&self) -> u64;
    fn time_since_boot(&self) -> Duration;
}
