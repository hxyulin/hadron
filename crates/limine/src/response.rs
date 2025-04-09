use core::{
    cell::UnsafeCell,
    ffi::{CStr, c_char},
    ptr::NonNull,
};

use crate::{
    file::{File, FileIter},
    framebuffer::{FramebufferIter, RawFramebuffer},
    memory_map::{MemoryMapEntry, MemoryMapIter},
};

#[repr(transparent)]
pub struct Response<T> {
    inner: UnsafeCell<Option<NonNull<T>>>,
}

unsafe impl<T> Sync for Response<T> {}
unsafe impl<T> Send for Response<T> {}

impl<T> Response<T> {
    pub const fn none() -> Self {
        Self {
            inner: UnsafeCell::new(None),
        }
    }

    pub fn get(&self) -> Option<&T> {
        Some(unsafe { core::ptr::read_volatile(self.inner.get())?.as_ref() })
    }
}

#[repr(C)]
pub struct BootloaderInfoResponse {
    pub revision: u64,
    name: *const c_char,
    version: *const c_char,
}

impl BootloaderInfoResponse {
    pub fn name(&self) -> &str {
        // SAFETY: The pointer is valid because it is a pointer to a string literal.
        unsafe { CStr::from_ptr(self.name).to_str().unwrap() }
    }

    pub fn version(&self) -> &str {
        // SAFETY: The pointer is valid because it is a pointer to a string literal.
        unsafe { CStr::from_ptr(self.version).to_str().unwrap() }
    }
}

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

#[repr(C)]
pub struct FirmwareTypeResponse {
    pub revision: u64,
    firmware_type: u64,
}

impl FirmwareTypeResponse {
    pub fn firmware_type(&self) -> FirmwareType {
        // SAFETY: The firmware type is a valid enum variant (ganranteed by the protocol).
        unsafe { core::mem::transmute(self.firmware_type) }
    }
}

#[repr(C)]
pub struct StackSizeResponse {
    pub revision: u64,
}

#[repr(C)]
pub struct HhdmResponse {
    pub revision: u64,
    pub offset: u64,
}

#[repr(C)]
pub struct FramebufferResponse {
    pub revision: u64,
    framebuffer_count: u64,
    framebuffers: NonNull<NonNull<RawFramebuffer>>,
}

impl FramebufferResponse {
    pub(crate) fn framebuffer_ptrs(&self) -> &[NonNull<RawFramebuffer>] {
        // SAFETY: The framebuffers pointer is valid because it is a pointer to an array of pointers.
        unsafe { core::slice::from_raw_parts(self.framebuffers.as_ptr(), self.framebuffer_count as usize) }
    }

    pub fn count(&self) -> usize {
        self.framebuffer_count as usize
    }

    pub fn framebuffers(&self) -> FramebufferIter {
        FramebufferIter::new(self.revision, self.framebuffer_ptrs())
    }
}

#[repr(C)]
pub struct PagingModeResponse {
    pub revision: u64,
    pub paging_mode: u64,
}

#[repr(C)]
pub struct MemoryMapResponse {
    pub revision: u64,
    pub memory_map_entries: u64,
    memory_map: NonNull<NonNull<MemoryMapEntry>>,
}

impl MemoryMapResponse {
    pub(crate) fn count(&self) -> usize {
        self.memory_map_entries as usize
    }

    pub fn memory_map(&self) -> MemoryMapIter {
        MemoryMapIter::new(unsafe { core::slice::from_raw_parts(self.memory_map.as_ptr(), self.count()) })
    }
}

#[repr(C)]
pub struct EntryPointResponse {
    pub revision: u64,
}

#[repr(C)]
pub struct ExecutableFileResponse {
    pub revision: u64,
    executable_file: NonNull<File>,
}

impl ExecutableFileResponse {
    pub fn executable_file(&self) -> &File {
        // SAFETY: The executable file pointer is valid because it is a pointer to a file.
        unsafe { self.executable_file.as_ref() }
    }
}

#[repr(C)]
pub struct ModuleResponse {
    pub revision: u64,
    modules_count: u64,
    modules: NonNull<NonNull<File>>,
}

impl ModuleResponse {
    pub(crate) fn count(&self) -> usize {
        self.modules_count as usize
    }

    pub fn modules(&self) -> FileIter {
        FileIter::new(self.revision, unsafe {
            core::slice::from_raw_parts(self.modules.as_ptr(), self.count())
        })
    }
}

#[repr(C)]
pub struct RsdpResponse {
    pub revision: u64,
    /// The address of the RSDP structure.
    /// Physical Address if the base revision >= 3
    pub address: u64,
}

#[repr(C)]
pub struct BootTimeResponse {
    pub revision: u64,
    /// The boot time in seconds since the UNIX epoch.
    pub boot_time: i64,
}

#[repr(C)]
pub struct ExecutableAddressResponse {
    pub revision: u64,
    /// The physical base address of the executable.
    pub physical_address: u64,
    /// The virtual base address of the executable.
    pub virtual_address: u64,
}
