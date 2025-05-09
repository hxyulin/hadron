use core::time::Duration;

use super::{Timer, TimerTicks};
use x86_64::VirtAddr;

use crate::base::io::mmio::OffsetMmio;

#[derive(Debug, Clone, Copy)]
pub struct HpetInfo {
    pub is_64bit: bool,
    pub num_comparators: u8,
    pub legacy_replacement: bool,
    pub minimum_tick: u64,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct Hpet {
    info: HpetInfo,
    addr: VirtAddr,

    capabilities: OffsetMmio<u64, 0x000>,
    config: OffsetMmio<u64, 0x010>,
    interrupt_status: OffsetMmio<u64, 0x020>,
    main_counter: OffsetMmio<u64, 0x0F0>,
    timer0_config: OffsetMmio<u64, 0x100>,
    timer0_comparator: OffsetMmio<u64, 0x108>,
    timer0_fsb_irration: OffsetMmio<u64, 0x110>,
}

impl Hpet {
    pub const SIZE: u32 = 0x400;
    pub const SIZE_ALIGNED: u64 = (Self::SIZE as u64 + 0xfff) & !0xfff;

    pub fn new(addr: VirtAddr, info: acpi::HpetInfo) -> Self {
        let info = HpetInfo {
            is_64bit: info.main_counter_is_64bits(),
            num_comparators: info.num_comparators(),
            legacy_replacement: info.legacy_irq_capable(),
            minimum_tick: info.clock_tick_unit as u64,
        };
        Self {
            info,
            addr,
            capabilities: OffsetMmio::default(),
            config: OffsetMmio::default(),
            interrupt_status: OffsetMmio::default(),
            main_counter: OffsetMmio::default(),
            timer0_config: OffsetMmio::default(),
            timer0_comparator: OffsetMmio::default(),
            timer0_fsb_irration: OffsetMmio::default(),
        }
    }

    /// Initializes the HPET.
    ///
    /// # Safety
    /// This function is unsafe because it can cause UB if the HPET is not valid, or aclled more than once.
    pub unsafe fn init(&mut self) {
        // Disable everything
        self.config.write(self.addr, 0);
        self.main_counter.write(self.addr, 0);
        self.interrupt_status.write(self.addr, u64::MAX);

        // Get capabilities
        let _capabilities = self.capabilities.read(self.addr);

        // Enable HPET
        let mut config = 0u64;
        config |= 1 << 0; // Enable
        if self.info.legacy_replacement {
            config |= 1 << 1; // Legacy replacement
        }
        self.config.write(self.addr, config);
    }

    pub fn read_counter(&self) -> u64 {
        self.main_counter.read(self.addr)
    }

    pub fn period(&self) -> u64 {
        let capabilities = self.capabilities.read(self.addr);
        (capabilities >> 32) & 0xFFFFFFFF
    }
}

impl Timer for Hpet {
    fn frequency(&self) -> u64 {
        let period = self.period();
        1_000_000_000_000_000 / period
    }

    fn now(&self) -> TimerTicks {
        TimerTicks::new(self.read_counter())
    }

    fn time_since_boot(&self) -> Duration {
        let now = self.read_counter();
        // Femto to nanoseconds conversion
        let ns = (now * self.period()) / 1_000_000;
        Duration::new(ns / 1_000_000_000, (ns % 1_000_000_000) as u32)
    }
}
