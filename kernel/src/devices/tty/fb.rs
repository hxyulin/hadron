use core::ops::DerefMut;

use noto_sans_mono_bitmap::RasterizedChar;

use crate::{
    base::info::kernel_info,
    devices::{
        DeviceId,
        fb::{BORDER_PADDING, FbDevice, LETTER_SPACING, LINE_SPACING, font_constants, get_char_raster},
    },
};

/// A TTY device that writes text to a virtual framebuffer
pub struct VirtFbTtyDevice {
    fb_id: DeviceId,
    writer: FbTtyWriter,
}

impl VirtFbTtyDevice {
    pub fn new(fb_id: DeviceId) -> Self {
        Self {
            fb_id,
            writer: FbTtyWriter::new(),
        }
    }

    pub fn set_pos(&mut self, x: u32, y: u32) {
        self.writer.x_pos = x as usize;
        self.writer.y_pos = y as usize;
    }
}

impl super::Device for VirtFbTtyDevice {
    fn class_id(&self) -> crate::devices::DeviceClass {
        crate::devices::DeviceClass::Tty
    }

    fn as_any(&self) -> &dyn core::any::Any {
        self
    }
}

impl super::TtyDevice for VirtFbTtyDevice {
    fn capabilities(&self) -> super::TtyDeviceCapabilities {
        super::TtyDeviceCapabilities::WRITE
    }

    fn write(&mut self, buf: &[u8]) -> usize {
        let fb = kernel_info().devices.get_fb_device(self.fb_id).unwrap();
        let mut fb = fb.lock();
        for c in buf {
            self.writer.write_char(fb.deref_mut(), *c as char);
        }
        buf.len()
    }
}

pub struct FbTtyWriter {
    x_pos: usize,
    y_pos: usize,
}

impl FbTtyWriter {
    pub fn new() -> Self {
        Self { x_pos: 0, y_pos: 0 }
    }

    fn newline(&mut self) {
        self.y_pos += font_constants::CHAR_RASTER_HEIGHT.val() + LINE_SPACING;
        self.carriage_return()
    }

    fn scroll_up(&mut self, fb: &mut dyn FbDevice) {
        let row_size = fb.info().stride as usize;
        let height = fb.info().height as usize;
        let line_height = font_constants::CHAR_RASTER_HEIGHT.val() + LINE_SPACING;

        // Copy each line upwards
        for y in line_height..height {
            let dest_start = (y - line_height) * row_size;
            let src_start = y * row_size;

            fb.raw_buffer()
                .copy_within(src_start..(src_start + row_size), dest_start);
        }

        // Clear the last line
        let last_line_start = (height - line_height) * row_size;
        fb.raw_buffer()[last_line_start..].fill(0);

        // Reset y_pos to stay at the last line
        self.y_pos = height - line_height;
    }

    fn carriage_return(&mut self) {
        self.x_pos = BORDER_PADDING;
    }

    /// Writes a single char to the framebuffer. Takes care of special control characters, such as
    /// newlines and carriage returns.
    pub fn write_char(&mut self, fb: &mut dyn FbDevice, c: char) {
        let width = fb.info().width as usize;
        match c {
            '\n' => self.newline(),
            '\r' => self.carriage_return(),
            '\t' => {
                self.x_pos += font_constants::CHAR_RASTER_WIDTH * 4;
                if self.x_pos >= width {
                    self.newline();
                }
            }
            c => {
                let new_xpos = self.x_pos + font_constants::CHAR_RASTER_WIDTH;
                if new_xpos >= width {
                    self.newline();
                }
                let new_ypos = self.y_pos + font_constants::CHAR_RASTER_HEIGHT.val();
                if new_ypos >= fb.info().height as usize {
                    self.scroll_up(fb);
                }
                self.write_rendered_char(fb, get_char_raster(c));
            }
        }
    }

    /// Prints a rendered char into the framebuffer.
    /// Updates `self.x_pos`.
    fn write_rendered_char(&mut self, fb: &mut dyn FbDevice, rendered_char: RasterizedChar) {
        for (y, row) in rendered_char.raster().iter().enumerate() {
            for (x, byte) in row.iter().enumerate() {
                let color = [*byte; 4];
                fb.write_pixel(
                    (self.x_pos + x) as u32,
                    (self.y_pos + y) as u32,
                    u32::from_le_bytes(color),
                );
            }
        }
        self.x_pos += rendered_char.width() + LETTER_SPACING;
    }
}
