//! QEMU qemu-vga driver
//!
//! Specification: [QEMU qemu-vga Device](https://www.qemu.org/docs/master/specs/standard-vga.html)

use x86_64::{
    PhysAddr, VirtAddr,
    structures::paging::{Page, PageSize, PageTableFlags, PhysFrame, Size4KiB},
};

use crate::{
    base::{
        io::mmio::OffsetMmio,
        mem::{map_page, mappings, sync::UninitCell},
    },
    dev::{DeviceFunctionInner, DeviceId, drivers::DriverVTable},
};

pub fn matches(device_id: &DeviceId) -> bool {
    device_id.vendor_id() == 0x1234 && device_id.device_id() == 0x1111
}

pub fn get_vtable() -> DriverVTable {
    DriverVTable { probe, init }
}

fn probe(device: &DeviceFunctionInner) -> bool {
    true
}

struct VgaDriver {
    mmio: MmioArea,
}

struct MmioArea {
    base: VirtAddr,

    // Bochs MMIO registers
    id: OffsetMmio<u16, 0x0500>,
    x_res: OffsetMmio<u16, 0x0502>,
    y_res: OffsetMmio<u16, 0x0504>,
    bpp: OffsetMmio<u16, 0x0506>,
    enable: OffsetMmio<u16, 0x0508>,
    bank: OffsetMmio<u16, 0x050A>,
    v_width: OffsetMmio<u16, 0x050C>,
    v_height: OffsetMmio<u16, 0x050E>,
    x_offset: OffsetMmio<u16, 0x0510>,
    y_offset: OffsetMmio<u16, 0x0512>,

    endianness: OffsetMmio<u32, 0x0604>,
}

#[derive(Debug, PartialEq, Eq)]
enum Endianness {
    Little,
    Big,
}

impl MmioArea {
    const INDEX_X_RES: u16 = 0x01;
    const INDEX_Y_RES: u16 = 0x02;

    pub const fn new(base: VirtAddr) -> Self {
        Self {
            base,

            id: OffsetMmio::new(),
            x_res: OffsetMmio::new(),
            y_res: OffsetMmio::new(),
            bpp: OffsetMmio::new(),
            enable: OffsetMmio::new(),
            bank: OffsetMmio::new(),
            v_width: OffsetMmio::new(),
            v_height: OffsetMmio::new(),
            x_offset: OffsetMmio::new(),
            y_offset: OffsetMmio::new(),

            endianness: OffsetMmio::new(),
        }
    }

    pub fn id(&self) -> u16 {
        self.id.read(self.base)
    }

    pub fn resolution(&self) -> (u16, u16, u16) {
        (
            self.x_res.read(self.base),
            self.y_res.read(self.base),
            self.bpp.read(self.base),
        )
    }

    pub fn set_resolution(&self, width: u16, height: u16, bpp: u16) {
        self.x_res.write(self.base, width);
        self.y_res.write(self.base, height);
        self.bpp.write(self.base, bpp);
    }

    pub fn virtual_size(&self) -> (u16, u16) {
        (self.v_width.read(self.base), self.v_height.read(self.base))
    }

    pub fn endianness(&self) -> Endianness {
        let endian = self.endianness.read(self.base);
        if endian == 0x1e1e1e1e {
            Endianness::Little
        } else if endian == 0xbebebebe {
            Endianness::Big
        } else {
            log::debug!("qemu-vga: unknown endianness: {:#x}, defaulting to littel", endian);
            Endianness::Little
        }
    }

    pub fn enabled(&self) -> u16 {
        self.enable.read(self.base)
    }

    pub fn set_enable(&self, enable: u16) {
        self.enable.write(self.base, enable);
    }
}

static VGA: UninitCell<VgaDriver> = UninitCell::uninit();

fn init(device: &DeviceFunctionInner) -> bool {
    log::debug!("initializing qemu-vga driver");
    let fb_addr = PhysAddr::new(device.bars[0] as u64);
    let mmio_addr = PhysAddr::new(device.bars[2] as u64);
    let mmio_virt = crate::base::io::mmio::allocate_persistent(mmio_addr, 0x1000);

    let mmio = MmioArea::new(mmio_virt);
    let id = mmio.id();
    if id != 0xB0C5 {
        log::debug!("qemu-vga: unknown ID: {:#x}", id);
        return false;
    }

    let endianness = mmio.endianness();
    log::debug!("qemu-vga: endianness: {:?}", endianness);
    // First we need to disable the qemu-vga
    mmio.set_enable(0);
    // Then we need to set the resolution
    // We want 800x600x32
    log::debug!("qemu-vga: requesting resolution 800x600x32");
    mmio.set_resolution(800, 600, 32);
    // Finally we need to enable the qemu-vga, with linear framebuffer
    mmio.set_enable(0x41);

    // We need to read from it to see what it actually set the resolution to
    let (width, height, bpp) = mmio.resolution();
    log::debug!("qemu-vga: resolution: {}x{}x{}", width, height, bpp);
    // We need ro read the virtual size to know how much memory we need to allocate, and the stride
    let (vsize_width, vsize_height) = mmio.virtual_size();
    log::debug!("qemu-vga: virtual size: {}x{}", vsize_width, vsize_height);

    let size: u64 = vsize_width as u64 * vsize_height as u64 * bpp as u64 / 8;
    let pages = size.div_ceil(Size4KiB::SIZE);
    for i in 0..pages {
        let offset = i * Size4KiB::SIZE;
        unsafe {
            map_page(
                PhysFrame::<Size4KiB>::from_start_address_unchecked(fb_addr + offset),
                mappings::FRAMEBUFFER + offset,
                PageTableFlags::PRESENT
                    | PageTableFlags::WRITABLE
                    | PageTableFlags::NO_EXECUTE
                    | PageTableFlags::NO_CACHE,
            )
        };
    }

    let driver = VgaDriver { mmio };
    core::mem::forget(unsafe { VGA.replace(driver) });
    true
}
