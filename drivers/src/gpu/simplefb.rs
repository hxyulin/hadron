use hadron_driver_api::platform::{PlatformDev, PlatformDriver, PlatformDrvMatch};

#[used]
#[unsafe(link_section = ".platform_drivers")]
static DRIVER: PlatformDriver = PlatformDriver {
    name: "Simple Framebuffer",
    probe,
    matches: &[PlatformDrvMatch {
        compatible: "simple-fb",
    }],
};

pub struct SimpleFbDev {
    pub width: u32,
    pub height: u32,
    pub bpp: u32,
    pub pitch: u32,
}

fn probe(dev: &PlatformDev) -> u32 {
    // Data contains is:
    // 0: FramebufferInfo
    // 1: PhysAddr (of the framebuffer)
    log::info!("Device name: {}", dev.name);
    0
}
