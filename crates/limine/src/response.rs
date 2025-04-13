//! Types for representing responses.

use core::{
    cell::UnsafeCell,
    ffi::{CStr, c_char},
    ptr::NonNull,
};

use crate::{
    file::{File, FileIter},
    framebuffer::{FramebufferList, RawFramebuffer},
    memory_map::{MemoryMapEntry, MemoryMapIter},
};

/// A response to a request.
#[repr(transparent)]
pub struct Response<T> {
    inner: UnsafeCell<Option<NonNull<T>>>,
}

unsafe impl<T> Sync for Response<T> {}
unsafe impl<T> Send for Response<T> {}

impl<T> Response<T> {
    /// Creates a new `Response` that does not contain a value.
    pub const fn none() -> Self {
        Self {
            inner: UnsafeCell::new(None),
        }
    }

    /// Returns the value of the response.
    pub fn get(&self) -> Option<&T> {
        Some(unsafe { core::ptr::read_volatile(self.inner.get())?.as_ref() })
    }
}

/// The response to the [`BootloaderInfoRequest`].
#[repr(C)]
pub struct BootloaderInfoResponse {
    pub revision: u64,
    name: *const c_char,
    version: *const c_char,
}

impl BootloaderInfoResponse {
    /// Returns the name of the bootloader.
    pub fn name(&self) -> &str {
        // SAFETY: The pointer is valid because it is a pointer to a string literal.
        unsafe { CStr::from_ptr(self.name).to_str().unwrap() }
    }

    /// Returns the version of the bootloader.
    pub fn version(&self) -> &str {
        // SAFETY: The pointer is valid because it is a pointer to a string literal.
        unsafe { CStr::from_ptr(self.version).to_str().unwrap() }
    }
}

/// The firmware type of the bootloader.
#[repr(u64)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FirmwareType {
    /// X86 BIOS
    X86BIOS = 0,
    /// UEFI 32-bit
    UEFI32 = 1,
    /// UEFI 64-bit
    UEFI64 = 2,
    /// SBI (Simple Boot Initiator)
    SBI = 3,
}

/// The response to the [`FirmwareTypeRequest`].
#[repr(C)]
pub struct FirmwareTypeResponse {
    pub revision: u64,
    firmware_type: u64,
}

impl FirmwareTypeResponse {
    /// Returns the firmware type of the bootloader.
    pub fn firmware_type(&self) -> FirmwareType {
        // SAFETY: The firmware type is a valid enum variant (ganranteed by the protocol).
        unsafe { core::mem::transmute(self.firmware_type) }
    }
}

/// The response to the [`StackSizeRequest`].
#[repr(C)]
pub struct StackSizeResponse {
    pub revision: u64,
}

/// The response to the [`HhdmRequest`].
#[repr(C)]
pub struct HhdmResponse {
    pub revision: u64,
    /// The offset of the Higher Half.
    pub offset: u64,
}

/// The response to the [`FramebufferRequest`].
#[repr(C)]
pub struct FramebufferResponse {
    pub revision: u64,
    /// The number of framebuffers.
    framebuffer_count: u64,
    /// The framebuffers.
    framebuffers: NonNull<NonNull<RawFramebuffer>>,
}

impl FramebufferResponse {
    /// Returns the framebuffer pointers.
    fn framebuffer_ptrs(&self) -> &[NonNull<RawFramebuffer>] {
        // SAFETY: The framebuffers pointer is valid because it is a pointer to an array of pointers.
        unsafe { core::slice::from_raw_parts(self.framebuffers.as_ptr(), self.framebuffer_count as usize) }
    }

    /// Returns the number of framebuffers.
    pub fn len(&self) -> usize {
        self.framebuffer_count as usize
    }

    /// Returns true if there are no framebuffers.
    pub fn is_empty(&self) -> bool {
        self.framebuffer_count == 0
    }

    /// Returns an iterator over the framebuffers.
    pub fn framebuffers(&self) -> FramebufferList {
        FramebufferList::new(self.revision, self.framebuffer_ptrs())
    }
}

/// The response to the [`PagingModeRequest`].
#[repr(C)]
pub struct PagingModeResponse {
    pub revision: u64,
    pub paging_mode: u64,
}

/// The response to the [`MemoryMapRequest`].
#[repr(C)]
pub struct MemoryMapResponse {
    pub revision: u64,
    pub memory_map_entries: u64,
    memory_map: NonNull<NonNull<MemoryMapEntry>>,
}

impl MemoryMapResponse {
    #[cfg(feature = "internal-api")]
    pub fn internal_new(revision: u64, memory_map_entries: u64, memory_map: NonNull<NonNull<MemoryMapEntry>>) -> Self {
        Self {
            revision,
            memory_map_entries,
            memory_map,
        }
    }

    /// Returns the number of memory map entries.
    pub(crate) fn count(&self) -> usize {
        self.memory_map_entries as usize
    }

    /// Returns an iterator over the memory map entries.
    pub fn entries(&self) -> MemoryMapIter {
        MemoryMapIter::new(unsafe { core::slice::from_raw_parts(self.memory_map.as_ptr(), self.count()) })
    }
}

/// The response to the [`EntryPointRequest`].
#[repr(C)]
pub struct EntryPointResponse {
    pub revision: u64,
}

/// The response to the [`ExecutableFileRequest`].
#[repr(C)]
pub struct ExecutableFileResponse {
    pub revision: u64,
    executable_file: NonNull<File>,
}

impl ExecutableFileResponse {
    /// Returns the executable file.
    pub fn executable_file(&self) -> &File {
        // SAFETY: The executable file pointer is valid because it is a pointer to a file.
        unsafe { self.executable_file.as_ref() }
    }
}

/// The response to the [`ModuleRequest`].
#[repr(C)]
pub struct ModuleResponse {
    pub revision: u64,
    modules_count: u64,
    modules: NonNull<NonNull<File>>,
}

impl ModuleResponse {
    /// Returns the number of modules.
    pub(crate) fn count(&self) -> usize {
        self.modules_count as usize
    }

    /// Returns an iterator over the modules.
    pub fn modules(&self) -> FileIter {
        // SAFETY: The modules pointer is valid because it is a pointer to an array of pointers.
        FileIter::new(unsafe { core::slice::from_raw_parts(self.modules.as_ptr(), self.count()) })
    }
}

/// The response to the [`RsdpRequest`].
#[repr(C)]
pub struct RsdpResponse {
    pub revision: u64,
    /// The address of the RSDP structure.
    /// Physical Address if the base revision >= 3
    pub address: u64,
}

/// The response to the [`BootTimeRequest`].
#[repr(C)]
pub struct BootTimeResponse {
    pub revision: u64,
    /// The boot time in seconds since the UNIX epoch.
    pub boot_time: i64,
}

/// The response to the [`ExecutableAddressRequest`].
#[repr(C)]
pub struct ExecutableAddressResponse {
    pub revision: u64,
    /// The physical base address of the executable.
    pub physical_address: u64,
    /// The virtual base address of the executable.
    pub virtual_address: u64,
}
