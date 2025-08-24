use crate::arch::x86_64::io::uart::Uart16550;
use alloc::{boxed::Box, sync::Arc};
use core::{ffi::c_void, ptr::NonNull};

use crate::dev::{
    Device, DeviceDriver,
    drivers::{
        ConsoleDevVTable, DriverCapabilities,
        platform::{PlatformDrv, PlatformDrvVTable},
    },
    platform::{PlatformDev, PlatformDevAddr, PlatformDevMatcher},
};
use spin::Mutex;

#[used]
#[unsafe(link_section = ".platform_drivers")]
static SERIAL_DRV: PlatformDrv = PlatformDrv {
    name: "Serial",
    vtable: PlatformDrvVTable { probe, attach },
    matchers: &[PlatformDevMatcher {
        name: "io_dev",
        addr: Some(PlatformDevAddr::io_port(0x3F8)),
    }],
    caps: DriverCapabilities {
        console: Some(&ConsoleDevVTable { write }),
        ..Default::default()
    },
};

fn probe(_dev: &PlatformDev) -> bool {
    true
}

fn attach(dev: &mut PlatformDev) {
    let mut serial = unsafe { Uart16550::new(dev.addr.io_port) };
    unsafe { serial.init() };
    let drv_data = Box::leak(Box::new(serial));
    let dev = Arc::get_mut(&mut dev.dev).expect("a driver can only be attached when the device is not referenced");
    dev.drv = NonNull::new(drv_data as *mut Uart16550 as *mut c_void).map(|data| DeviceDriver {
        data: Mutex::new(data),
        caps: &SERIAL_DRV.caps,
    });
}

fn write(dev: &Device, byte: u8) {
    let serial = unsafe { dev.drv.as_ref().unwrap().data.lock().cast::<Uart16550>().as_mut() };
    serial.write_byte(byte);
}
