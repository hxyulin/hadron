use super::Device;
use noto_sans_mono_bitmap::{FontWeight, RasterHeight, RasterizedChar, get_raster, get_raster_width};
use volatile::slice::VolatileSlice;

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

pub struct FbInfo {
    pub width: u32,
    pub height: u32,
    pub stride: u32,
}

pub trait FbDevice: Device {
    fn info(&self) -> FbInfo;
    fn write_pixel(&mut self, x: u32, y: u32, color: u32);
    fn raw_buffer(&mut self) -> &mut VolatileSlice<u8>;
}

impl super::Device for Framebuffer {
    fn class_id(&self) -> crate::devices::DeviceClass {
        crate::devices::DeviceClass::Framebuffer
    }

    fn as_any(&self) -> &dyn core::any::Any {
        self
    }
}

impl FbDevice for Framebuffer {
    fn info(&self) -> FbInfo {
        FbInfo {
            width: self.info.width,
            height: self.info.height,
            stride: self.info.stride,
        }
    }

    fn write_pixel(&mut self, x: u32, y: u32, color: u32) {
        self.write_pixel(x, y, color);
    }

    fn raw_buffer(&mut self) -> &mut VolatileSlice<u8> {
        self.buffer
    }
}
