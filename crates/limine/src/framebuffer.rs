//! Types for representing framebuffers.

use core::ptr::NonNull;

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum MemoryModel {
    Rgb = 0,
}

/// A framebuffer wrapper.
///
/// This wraps the `RawFramebuffer` to allow safe access to the framebuffer,
/// taking into account the revision of the protocol.
pub struct Framebuffer<'a> {
    /// The revision of the framebuffer
    revision: u64,
    /// The inner framebuffer
    inner: &'a RawFramebuffer,
}

impl<'a> Framebuffer<'a> {
    /// Creates a new `Framebuffer` from a raw framebuffer.
    pub(crate) fn new(revision: u64, inner: &'a RawFramebuffer) -> Self {
        Self { revision, inner }
    }

    /// Returns the virtual address of the framebuffer.
    /// If using the HHDM feature, the physical address is just the virtual address subtracted by the offset.
    pub fn address(&self) -> u64 {
        unsafe { self.inner.r0.address }
    }

    /// Returns the width of the framebuffer in pixels.
    pub fn width(&self) -> u64 {
        unsafe { self.inner.r0.width }
    }

    /// Returns the height of the framebuffer in pixels.
    pub fn height(&self) -> u64 {
        unsafe { self.inner.r0.height }
    }

    /// Returns the pitch of the framebuffer in bytes.
    /// (The number of bytes per scanline.)
    pub fn pitch(&self) -> u64 {
        unsafe { self.inner.r0.pitch }
    }

    /// Returns the number of bits per pixel.
    pub fn bpp(&self) -> u16 {
        unsafe { self.inner.r0.bpp }
    }

    /// Returns the memory model of the framebuffer.
    pub fn memory_model(&self) -> MemoryModel {
        // SAFETY: The memory model is a valid enum variant (guaranteed by the protocol).
        unsafe { core::mem::transmute(self.inner.r0.memory_model) }
    }

    /// Returns the size of the red mask in bits.
    pub fn red_mask_size(&self) -> u8 {
        unsafe { self.inner.r0.red_mask_size }
    }

    /// Returns the shift of the red mask in bits.
    pub fn red_mask_shift(&self) -> u8 {
        unsafe { self.inner.r0.red_mask_shift }
    }

    /// Returns the size of the green mask in bits.
    pub fn green_mask_size(&self) -> u8 {
        unsafe { self.inner.r0.green_mask_size }
    }

    /// Returns the shift of the green mask in bits.
    pub fn green_mask_shift(&self) -> u8 {
        unsafe { self.inner.r0.green_mask_shift }
    }

    /// Returns the size of the blue mask in bits.
    pub fn blue_mask_size(&self) -> u8 {
        unsafe { self.inner.r0.blue_mask_size }
    }

    /// Returns the shift of the blue mask in bits.
    pub fn blue_mask_shift(&self) -> u8 {
        unsafe { self.inner.r0.blue_mask_shift }
    }

    // TODO: Video Mode Iter
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
            .field("memory_model", &self.memory_model())
            .field("red_mask_size", &self.red_mask_size())
            .field("red_mask_shift", &self.red_mask_shift())
            .field("green_mask_size", &self.green_mask_size())
            .field("green_mask_shift", &self.green_mask_shift())
            .field("blue_mask_size", &self.blue_mask_size())
            .field("blue_mask_shift", &self.blue_mask_shift())
            .finish()
    }
}

/// A raw framebuffer.
/// This is a union of the different revisions of the framebuffer.
/// Only used internally, and not exposed to the user.
#[repr(C)]
pub(crate) union RawFramebuffer {
    /// Revision 0 of the framebuffer.
    r0: FramebufferR0,
    /// Revision 1 of the framebuffer.
    r1: FramebufferR1,
}

/// Framebuffer revision 0.
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

/// Framebuffer revision 1.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct FramebufferR1 {
    r0: FramebufferR0,
    mode_count: u64,
    modes: NonNull<NonNull<VideoMode>>,
}

/// A video mode.
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

/// An iterator over the framebuffers.
pub struct FramebufferIter<'a> {
    revision: u64,
    framebuffers: &'a [NonNull<RawFramebuffer>],
    index: usize,
}

impl<'a> FramebufferIter<'a> {
    /// Creates a new `FramebufferIter` from a slice of framebuffer pointers.
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
