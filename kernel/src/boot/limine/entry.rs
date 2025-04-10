use kernel_base::VirtAddr;

use super::requests;
use crate::boot::drivers::framebuffer::{Framebuffer, FramebufferInfo, FramebufferWriter, PixelFormat};
use crate::boot::limine::requests::EXECUTABLE_ADDRESS;
use crate::serial::SerialPort;

#[unsafe(no_mangle)]
pub extern "C" fn kernel_entry() -> ! {
    use core::fmt::Write;
    let mut serial = unsafe { SerialPort::new(0x3F8) };
    assert!(
        requests::BASE_REVISION.is_supported(),
        "Limine Base Revision {} is not supported",
        requests::BASE_REVISION.revision()
    );
    let response = requests::BOOTLOADER_INFO.get_response().unwrap();
    writeln!(serial, "Booted from {} {}", response.name(), response.version()).unwrap();
    let mut framebuffer = init_framebuffer();
    let mut writer = FramebufferWriter::new(&mut framebuffer);
    writeln!(writer, "Hello, world!").unwrap();

    let response = EXECUTABLE_ADDRESS.get_response().unwrap();
    writeln!(
        writer,
        "Kernel loaded at: {:?}",
        VirtAddr::new(response.virtual_address)
    )
    .unwrap();

    writeln!(writer, "Initializing GDT...").unwrap();
    crate::base::arch::init_gdt();

    panic!("Kernel entry point reached");
}

fn init_framebuffer() -> Framebuffer {
    let framebuffer = requests::FRAMEBUFFER
        .get_response()
        .unwrap()
        .framebuffers()
        .next()
        .unwrap();
    let fb_info = FramebufferInfo {
        width: framebuffer.width() as u32,
        height: framebuffer.height() as u32,
        stride: framebuffer.pitch() as u32,
        bpp: framebuffer.bpp() as u32 / 8,
        pixel_format: PixelFormat::RGB,
    };
    let size = fb_info.stride as usize * fb_info.height as usize;
    let buf = unsafe { core::slice::from_raw_parts_mut(framebuffer.address() as *mut u8, size) };
    Framebuffer::new(fb_info, buf)
}
