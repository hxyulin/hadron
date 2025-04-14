pub mod hpet;
pub mod rtc;

pub use core::time::Duration;

use alloc::boxed::Box;
use rtc::UtcTime;
use spin::RwLock;

pub struct TimerTicks(pub u64);

impl TimerTicks {
    pub const fn new(ticks: u64) -> Self {
        Self(ticks)
    }
}

pub fn time_since_boot() -> Duration {
    TIMER.read().as_ref().map(|t| t.time_since_boot()).unwrap_or_default()
}

pub fn current_time() -> UtcTime {
    rtc::read_time()
}

pub trait Timer: Send + Sync {
    fn now(&self) -> TimerTicks;
    fn frequency(&self) -> u64;
    fn time_since_boot(&self) -> Duration;
}

pub static TIMER: RwLock<Option<Box<dyn Timer>>> = RwLock::new(None);
