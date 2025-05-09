use x86_64::{VirtAddr, registers::model_specific::Msr};

use crate::base::io::mmio::{OffsetMmio, OffsetMmioArray};

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct LocalApic {
    base: VirtAddr,

    id: OffsetMmio<u32, 0x20>,
    version: OffsetMmio<u32, 0x30>,
    tpr: OffsetMmio<u32, 0x80>,
    apr: OffsetMmio<u32, 0x90>,
    ppr: OffsetMmio<u32, 0xa0>,
    eoi: OffsetMmio<u32, 0xb0>,
    rrd: OffsetMmio<u32, 0xc0>,
    ldr: OffsetMmio<u32, 0xd0>,
    sivr: OffsetMmio<u32, 0xf0>,
    isrs: OffsetMmioArray<u32, 0x100, 8>,
    tmrs: OffsetMmioArray<u32, 0x180, 8>,
    irrs: OffsetMmioArray<u32, 0x200, 8>,
    esr: OffsetMmio<u32, 0x280>,
    icrs: OffsetMmioArray<u32, 0x300, 2>,
    lvt_timer: OffsetMmio<u32, 0x320>,
    lvt_thermal_sensor: OffsetMmio<u32, 0x330>,
    lvt_perf_monitor: OffsetMmio<u32, 0x340>,
    lvt_lint0: OffsetMmio<u32, 0x350>,
    lvt_lint1: OffsetMmio<u32, 0x360>,
    lvt_error: OffsetMmio<u32, 0x370>,
    timer_initial_count: OffsetMmio<u32, 0x380>,
    timer_current_count: OffsetMmio<u32, 0x390>,
    timer_divide_config: OffsetMmio<u32, 0x3e0>,
}

impl LocalApic {
    pub const SIZE: u32 = 0x400;
    pub const SIZE_ALIGNED: u64 = (Self::SIZE as u64 + 0xfff) & !0xfff;

    pub fn new(base: VirtAddr) -> Self {
        Self {
            base,
            id: OffsetMmio::default(),
            version: OffsetMmio::default(),
            tpr: OffsetMmio::default(),
            apr: OffsetMmio::default(),
            ppr: OffsetMmio::default(),
            eoi: OffsetMmio::default(),
            rrd: OffsetMmio::default(),
            ldr: OffsetMmio::default(),
            sivr: OffsetMmio::default(),
            isrs: OffsetMmioArray::default(),
            tmrs: OffsetMmioArray::default(),
            irrs: OffsetMmioArray::default(),
            esr: OffsetMmio::default(),
            icrs: OffsetMmioArray::default(),
            lvt_timer: OffsetMmio::default(),
            lvt_thermal_sensor: OffsetMmio::default(),
            lvt_perf_monitor: OffsetMmio::default(),
            lvt_lint0: OffsetMmio::default(),
            lvt_lint1: OffsetMmio::default(),
            lvt_error: OffsetMmio::default(),
            timer_initial_count: OffsetMmio::default(),
            timer_current_count: OffsetMmio::default(),
            timer_divide_config: OffsetMmio::default(),
        }
    }

    pub fn eoi(&mut self) {
        self.eoi.write(self.base, 0);
    }

    fn check_support(&self) -> bool {
        // TODO: Use CPUID to check for APIC support
        true
    }

    pub fn init(&mut self, phys_base: u64) -> bool {
        let has_apic = self.check_support();
        if !has_apic {
            return false;
        }

        unsafe { Msr::new(0x1B).write(phys_base | (1 << 11)) };

        let value = self.sivr.read(self.base);
        self.sivr.write(self.base, value | 0x100);
        true
    }
}
