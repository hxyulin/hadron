use super::requests::*;
use crate::{
    boot::drivers::framebuffer::{Framebuffer, FramebufferInfo, FramebufferWriter, PixelFormat},
    serial::SerialPort,
};

#[unsafe(no_mangle)]
pub extern "C" fn kernel_entry() -> ! {
    use core::fmt::Write;
    let mut serial = unsafe { SerialPort::new(0x3F8) };
    if !BASE_REVISION.is_supported() {
        panic!("Limine Base Revision ({}) is not supported", BASE_REVISION.revision());
    }
    let response = BOOTLOADER_INFO.get_response().unwrap();
    writeln!(serial, "Booted from {} {}", response.name(), response.version()).unwrap();
    let framebuffer = FRAMEBUFFER.get_response().unwrap().framebuffers().next().unwrap();
    let fb_info = FramebufferInfo {
        width: framebuffer.width() as u32,
        height: framebuffer.height() as u32,
        stride: framebuffer.pitch() as u32,
        bpp: framebuffer.bpp() as u32 / 8,
        pixel_format: PixelFormat::RGB,
    };
    let size = fb_info.stride as usize * fb_info.height as usize;
    let buf = unsafe { core::slice::from_raw_parts_mut(framebuffer.address() as *mut u8, size) };
    let mut framebuffer = Framebuffer::new(fb_info, buf);
    let mut writer = FramebufferWriter::new(&mut framebuffer);
    writer.write_str("Hello, world!\n");

    panic!("Kernel entry point reached");
}
