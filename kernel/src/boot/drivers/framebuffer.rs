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
        self.fb.fb_addr()
    }

    pub fn fb_size(&self) -> usize {
        self.fb.size()
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
