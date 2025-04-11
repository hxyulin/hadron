use volatile::slice::VolatileSlice;

use crate::devices::fb::{BORDER_PADDING, Framebuffer, LETTER_SPACING, LINE_SPACING, font_constants, get_char_raster};
use noto_sans_mono_bitmap::RasterizedChar;

pub struct FramebufferWriter {
    fb: Framebuffer,
    inner: FramebufferWriterInner,
}

impl FramebufferWriter {
    pub fn new(fb: Framebuffer) -> Self {
        Self {
            fb,
            inner: FramebufferWriterInner::new(),
        }
    }

    pub fn fb_addr(&self) -> usize {
        self.fb.buffer.as_ptr() as usize
    }

    /// Sets the framebuffer address.
    ///
    /// # Safety
    /// This function is unsafe because it can cause UB if the address is not valid.
    pub unsafe fn set_fb_addr(&mut self, addr: usize) {
        let slice = unsafe { core::slice::from_raw_parts_mut(addr as *mut u8, self.fb.buffer.len()) };
        self.fb.buffer = VolatileSlice::from_slice_mut(slice);
    }

    pub fn fb_size(&self) -> usize {
        self.fb.buffer.len()
    }

    pub fn to_inner(self) -> (Framebuffer, FramebufferWriterInner) {
        (self.fb, self.inner)
    }
}

impl core::fmt::Write for FramebufferWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            self.inner.write_char(&mut self.fb, c);
        }
        Ok(())
    }
}

pub struct FramebufferWriterInner {
    x_pos: usize,
    y_pos: usize,
}

impl FramebufferWriterInner {
    pub fn new() -> Self {
        Self {
            x_pos: BORDER_PADDING,
            y_pos: BORDER_PADDING,
        }
    }

    pub fn x_pos(&self) -> usize {
        self.x_pos
    }

    pub fn y_pos(&self) -> usize {
        self.y_pos
    }

    fn newline(&mut self) {
        self.y_pos += font_constants::CHAR_RASTER_HEIGHT.val() + LINE_SPACING;
        self.carriage_return()
    }

    fn scroll_up(&mut self, fb: &mut Framebuffer) {
        let row_size = fb.info.stride as usize;
        let height = fb.info.height as usize;
        let line_height = font_constants::CHAR_RASTER_HEIGHT.val() + LINE_SPACING;

        // Copy each line upwards
        for y in line_height..height {
            let dest_start = (y - line_height) * row_size;
            let src_start = y * row_size;

            fb.buffer.copy_within(src_start..(src_start + row_size), dest_start);
        }

        // Clear the last line
        let last_line_start = (height - line_height) * row_size;
        fb.buffer[last_line_start..].fill(0);

        // Reset y_pos to stay at the last line
        self.y_pos = height - line_height;
    }

    fn carriage_return(&mut self) {
        self.x_pos = BORDER_PADDING;
    }

    /// Writes a single char to the framebuffer. Takes care of special control characters, such as
    /// newlines and carriage returns.
    pub fn write_char(&mut self, fb: &mut Framebuffer, c: char) {
        let width = fb.info.width as usize;
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
                if new_ypos >= fb.info.height as usize {
                    self.scroll_up(fb);
                }
                self.write_rendered_char(fb, get_char_raster(c));
            }
        }
    }

    /// Prints a rendered char into the framebuffer.
    /// Updates `self.x_pos`.
    fn write_rendered_char(&mut self, fb: &mut Framebuffer, rendered_char: RasterizedChar) {
        for (y, row) in rendered_char.raster().iter().enumerate() {
            for (x, byte) in row.iter().enumerate() {
                self.write_pixel(fb, self.x_pos + x, self.y_pos + y, *byte);
            }
        }
        self.x_pos += rendered_char.width() + LETTER_SPACING;
    }

    fn write_pixel(&mut self, fb: &mut Framebuffer, x: usize, y: usize, intensity: u8) {
        let bpp = fb.info.bpp as usize;
        let byte_offset = y * fb.info.stride as usize + (x * bpp);
        let color = [intensity; 4];
        fb.buffer[byte_offset..(byte_offset + bpp)].copy_from_slice(&color[..bpp]);
    }
}
