use crate::dev::{
    drivers::DriverCapabilities,
    platform::{PlatformDev, PlatformDevMatcher},
};

pub mod fb;
#[cfg(target_arch = "x86_64")]
pub mod serial;

#[repr(C)]
#[derive(Debug)]
pub struct PlatformDrv {
    pub name: &'static str,
    pub matchers: &'static [PlatformDevMatcher],
    pub vtable: PlatformDrvVTable,
    pub caps: DriverCapabilities,
}

impl PlatformDrv {
    pub fn matches(&self, dev: &PlatformDev) -> bool {
        self.matchers.iter().find(|m| m.matches(dev)).is_some()
    }

    pub fn probe(&self, dev: &PlatformDev) -> bool {
        (self.vtable.probe)(dev)
    }

    pub fn attach(&self, dev: &mut PlatformDev) {
        (self.vtable.attach)(dev)
    }
}

#[repr(C)]
pub struct PlatformDrvVTable {
    probe: fn(&PlatformDev) -> bool,
    attach: fn(&mut PlatformDev),
}

impl core::fmt::Debug for PlatformDrvVTable {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PlatformDrvVTable")
            .field("probe", &format_args!("{:#x}", self.probe as usize))
            .finish()
    }
}

/// List the Available Platform Drivers
pub fn available_drivers() -> &'static [PlatformDrv] {
    unsafe extern "C" {
        static _platform_drv_start: u8;
        static _platform_drv_end: u8;
    }
    let size = (&raw const _platform_drv_end) as usize - (&raw const _platform_drv_start) as usize;
    unsafe {
        core::slice::from_raw_parts(
            (&raw const _platform_drv_start).cast::<PlatformDrv>(),
            size / size_of::<PlatformDrv>(),
        )
    }
}
