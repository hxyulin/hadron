use noto_sans_mono_bitmap::{FontWeight, RasterHeight, RasterizedChar, get_raster, get_raster_width};
use volatile::slice::VolatileSlice;
use x86_64::{
    VirtAddr,
    structures::paging::{Page, PageSize, Size4KiB},
};

use super::{Writer, WriterType};

pub struct FramebufferWriter {
    fb: Framebuffer,
    inner: FramebufferWriterInner,
}

impl Writer for FramebufferWriter {
    fn get_type(&self) -> WriterType {
        WriterType::Framebuffer
    }
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

/// Additional vertical space between lines
pub const LINE_SPACING: usize = 1;
/// Additional horizontal space between characters.
pub const LETTER_SPACING: usize = 0;

/// Padding from the border. Prevent that font is too close to border.
pub const BORDER_PADDING: usize = 1;

/// Constants for the usage of the [`noto_sans_mono_bitmap`] crate.
pub mod font_constants {
    use super::*;

    /// Height of each char raster. The font size is ~0.84% of this. Thus, this is the line height that
    /// enables multiple characters to be side-by-side and appear optically in one line in a natural way.
    pub const CHAR_RASTER_HEIGHT: RasterHeight = RasterHeight::Size16;

    /// The width of each single symbol of the mono space font.
    pub const CHAR_RASTER_WIDTH: usize = get_raster_width(FontWeight::Regular, CHAR_RASTER_HEIGHT);

    /// Backup character if a desired symbol is not available by the font.
    /// The '�' character requires the feature "unicode-specials".
    pub const BACKUP_CHAR: char = '�';

    pub const FONT_WEIGHT: FontWeight = FontWeight::Regular;
}

/// Returns the raster of the given char or the raster of [`font_constants::BACKUP_CHAR`].
pub fn get_char_raster(c: char) -> RasterizedChar {
    fn get(c: char) -> Option<RasterizedChar> {
        get_raster(c, font_constants::FONT_WEIGHT, font_constants::CHAR_RASTER_HEIGHT)
    }
    get(c).unwrap_or_else(|| get(font_constants::BACKUP_CHAR).expect("Should get raster of backup char."))
}

#[derive(Debug, Clone, Copy)]
pub enum PixelFormat {
    RGB,
}

#[derive(Debug)]
pub struct FramebufferInfo {
    pub width: u32,
    pub height: u32,
    pub pixel_format: PixelFormat,
    pub stride: u32,
    /// Bytes per pixel
    pub bpp: u32,
}

#[derive(Debug)]
pub struct Framebuffer {
    pub info: FramebufferInfo,
    pub buffer: &'static mut VolatileSlice<u8>,
}

impl Framebuffer {
    pub fn new(info: FramebufferInfo, buffer: &'static mut [u8]) -> Self {
        Self {
            info,
            buffer: VolatileSlice::from_slice_mut(buffer),
        }
    }

    pub fn write_pixel(&mut self, x: u32, y: u32, color: u32) {
        let offset = (y * self.info.stride + x * self.info.bpp) as usize;
        match self.info.pixel_format {
            PixelFormat::RGB => {
                self.buffer[offset].set((color & 0xFF) as u8);
                self.buffer[offset + 1].set(((color >> 8) & 0xFF) as u8);
                self.buffer[offset + 2].set(((color >> 16) & 0xFF) as u8);
            }
        }
    }

    pub fn fill(&mut self, color: u32) {
        for y in 0..self.info.height {
            for x in 0..self.info.width {
                self.write_pixel(x, y, color);
            }
        }
    }
}

impl Drop for FramebufferWriter {
    fn drop(&mut self) {
        // TODO: We need to be careful we don't panic here, otherwise we can't print the panic info
        use crate::base::mem::page_table::PageTable;
        let addr = VirtAddr::new(self.fb_addr() as u64);
        let pages = (self.fb_size() as u64).div_ceil(Size4KiB::SIZE);
        let mut page_table = crate::base::mem::PAGE_TABLE.lock();
        for i in 0..pages {
            unsafe {
                page_table.unmap(Page::<Size4KiB>::from_start_address_unchecked(
                    addr + i * Size4KiB::SIZE,
                ))
            };
        }
    }
}
