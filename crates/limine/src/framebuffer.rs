use core::ptr::NonNull;

pub struct FramebufferIter<'a> {
    revision: u64,
    framebuffers: &'a [NonNull<RawFramebuffer>],
    index: usize,
}

impl<'a> FramebufferIter<'a> {
    pub(crate) fn new(revision: u64, framebuffers: &'a [NonNull<RawFramebuffer>]) -> Self {
        Self {
            revision,
            framebuffers,
            index: 0,
        }
    }
}

impl<'a> Iterator for FramebufferIter<'a> {
    type Item = Framebuffer<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.framebuffers.as_ref().len() {
            return None;
        }
        self.index += 1;
        // SAFETY: The framebuffer pointer is valid because it is a pointer to a framebuffer.
        Some(Framebuffer::new(self.revision, unsafe {
            self.framebuffers[self.index - 1].as_ref()
        }))
    }
}

pub struct Framebuffer<'a> {
    revision: u64,
    inner: &'a RawFramebuffer,
}

impl<'a> Framebuffer<'a> {
    pub(crate) fn new(revision: u64, inner: &'a RawFramebuffer) -> Self {
        Self { revision, inner }
    }

    pub fn address(&self) -> u64 {
        unsafe { self.inner.r0.address }
    }

    pub fn width(&self) -> u64 {
        unsafe { self.inner.r0.width }
    }

    pub fn height(&self) -> u64 {
        unsafe { self.inner.r0.height }
    }

    pub fn pitch(&self) -> u64 {
        unsafe { self.inner.r0.pitch }
    }

    pub fn bpp(&self) -> u16 {
        unsafe { self.inner.r0.bpp }
    }
}

impl core::fmt::Debug for Framebuffer<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Framebuffer")
            .field("revision", &self.revision)
            .field("address", &self.address())
            .field("width", &self.width())
            .field("height", &self.height())
            .field("pitch", &self.pitch())
            .field("bpp", &self.bpp())
            .finish()
    }
}

#[repr(C)]
pub(crate) union RawFramebuffer {
    r0: FramebufferR0,
    r1: FramebufferR1,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct FramebufferR0 {
    address: u64,
    width: u64,
    height: u64,
    pitch: u64,
    /// The number of bits per pixel.
    bpp: u16,
    memory_model: u8,
    red_mask_size: u8,
    red_mask_shift: u8,
    green_mask_size: u8,
    green_mask_shift: u8,
    blue_mask_size: u8,
    blue_mask_shift: u8,
    unused: [u8; 7],
    edid_size: u64,
    edid_ptr: u64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct FramebufferR1 {
    r0: FramebufferR0,
    mode_count: u64,
    modes: NonNull<NonNull<VideoMode>>,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VideoMode {
    pub pitch: u64,
    pub width: u64,
    pub height: u64,
    pub bpp: u64,
    pub memory_model: u8,
    pub red_mask_size: u8,
    pub red_mask_shift: u8,
    pub green_mask_size: u8,
    pub green_mask_shift: u8,
    pub blue_mask_size: u8,
    pub blue_mask_shift: u8,
}
