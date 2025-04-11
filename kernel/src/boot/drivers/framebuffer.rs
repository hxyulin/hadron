use volatile::slice::VolatileSlice;

use crate::devices::framebuffer::{Framebuffer, FramebufferWriterInner};

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
