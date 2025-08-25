//! Types for representing requests.

use core::ptr::NonNull;

use crate::{
    module::InternalModule,
    response::{
        BootTimeResponse, BootloaderInfoResponse, EfiMemoryMapResponse, EfiSystemTableResponse, EntryPointResponse,
        ExecutableAddressResponse, ExecutableFileResponse, FirmwareTypeResponse, FramebufferResponse, HhdmResponse,
        MemoryMapResponse, ModuleResponse, MultiprocessorResponse, PagingModeResponse, Response, RsdpResponse,
        SmBiosResponse, StackSizeResponse,
    },
};

/// Given the request specific ID, returns the magic number containing the common magic number and the request specific ID.
macro_rules! request_magic {
    ($p3:literal, $p4:literal) => {
        [0xc7b1dd30df4c8b88, 0x0a82e883a194f07b, $p3, $p4]
    };
}

macro_rules! request_boilerplate {
    ($ty:ty) => {
        /// Returns the response of the request.
        pub fn response(&self) -> Option<&$ty> {
            self.response.get()
        }
    };
}

/// The start marker of the requests section.
///
/// This should be placed at the start of the requests section.
/// To ganrantee that the section is placed first, it should be placed at the start of the `.requests_start_marker` section,
/// which can be placed before the `.requests` section in the linker script.
#[repr(transparent)]
pub struct RequestsStartMarker {
    magic: [u64; 4],
}

impl RequestsStartMarker {
    /// Creates a new start marker.
    pub const fn new() -> Self {
        Self {
            magic: [
                0xf6b8f4b39de7d1ae,
                0xfab91a6940fcb9cf,
                0x785c6ed015d3e316,
                0x181e920a7852b9d9,
            ],
        }
    }
}

/// The end marker of the requests section.
///
/// This should be placed at the end of the requests section.
/// To ganrantee that the section is placed last, it should be placed at the end of the `.requests_end_marker` section,
/// which can be placed after the `.requests` section in the linker script.
#[repr(transparent)]
pub struct RequestsEndMarker {
    magic: [u64; 2],
}

impl RequestsEndMarker {
    /// Creates a new end marker.
    pub const fn new() -> Self {
        Self {
            magic: [0xadc0e0531bb10d03, 0x9572709f31764c62],
        }
    }
}

/// A request to get the bootloader information.
/// This returns the name, and version of the bootloader.
#[repr(C)]
pub struct BootloaderInfoRequest {
    id: [u64; 4],
    revision: u64,
    response: Response<BootloaderInfoResponse>,
}

impl BootloaderInfoRequest {
    request_boilerplate!(BootloaderInfoResponse);
    pub const LATEST_REVISION: u64 = 0;

    /// Creates a new request.
    pub const fn new() -> Self {
        Self {
            id: request_magic!(0xf55038d8e2a1202f, 0x279426fcf5f59740),
            revision: Self::LATEST_REVISION,
            response: Response::none(),
        }
    }
}

/// A request to get the firmware type.
/// This can tell you if the bootloader is UEFI or BIOS.
#[repr(C)]
pub struct FirmwareTypeRequest {
    id: [u64; 4],
    revision: u64,
    response: Response<FirmwareTypeResponse>,
}

impl FirmwareTypeRequest {
    pub const LATEST_REVISION: u64 = 0;
    request_boilerplate!(FirmwareTypeResponse);

    /// Creates a new request.
    pub const fn new() -> Self {
        Self {
            id: request_magic!(0x8c2f75d90bef28a8, 0x7045a4688eac00c3),
            revision: Self::LATEST_REVISION,
            response: Response::none(),
        }
    }
}

/// A request to set the stack size.
/// This is used to set the stack size of the kernel.
#[repr(C)]
pub struct StackSizeRequest {
    id: [u64; 4],
    revision: u64,
    response: Response<StackSizeResponse>,
    stack_size: u64,
}

impl StackSizeRequest {
    pub const LATEST_REVISION: u64 = 0;
    request_boilerplate!(StackSizeResponse);

    /// Creates a new request.
    ///
    /// The stack size is in bytes.
    pub const fn new(stack_size: u64) -> Self {
        Self {
            id: request_magic!(0x224ef0460a8e8926, 0xe1cb0fc25f46ea3d),
            revision: Self::LATEST_REVISION,
            response: Response::none(),
            stack_size,
        }
    }
}

/// A request to request the Higher Half Direct Memory feature.
#[repr(C)]
pub struct HhdmRequest {
    id: [u64; 4],
    revision: u64,
    response: Response<HhdmResponse>,
}

impl HhdmRequest {
    pub const LATEST_REVISION: u64 = 0;
    request_boilerplate!(HhdmResponse);

    /// Creates a new request.
    pub const fn new() -> Self {
        Self {
            id: request_magic!(0x48dcf1cb8ad2b852, 0x63984e959a98244b),
            revision: Self::LATEST_REVISION,
            response: Response::none(),
        }
    }
}

/// A request to get the framebuffers.
#[repr(C)]
pub struct FramebufferRequest {
    id: [u64; 4],
    revision: u64,
    response: Response<FramebufferResponse>,
}

impl FramebufferRequest {
    pub const LATEST_REVISION: u64 = 1;
    request_boilerplate!(FramebufferResponse);

    /// Creates a new request.
    pub const fn new() -> Self {
        Self::with_revision(Self::LATEST_REVISION)
    }

    /// Creates a new request with the given revision.
    pub const fn with_revision(revision: u64) -> Self {
        Self {
            id: request_magic!(0x9d5827dcd881dd75, 0xa3148604f6fab11b),
            revision,
            response: Response::none(),
        }
    }
}

/// Requests the current paging mode.
/// This is not completely implemented or supported, refer to the specification for more information on how to use this request.
#[repr(C)]
pub struct PagingModeRequest {
    id: [u64; 4],
    revision: u64,
    response: Response<PagingModeResponse>,
    paging_mode: u64,
    /// Only in revision 1+.
    max_mode: u64,
    /// Only in revision 1+.
    min_mode: u64,
}

impl PagingModeRequest {
    pub const LATEST_REVISION: u64 = 1;
    request_boilerplate!(PagingModeResponse);

    /// Creates a new request.
    pub const fn new(paging_mode: u64, max_mode: u64, min_mode: u64) -> Self {
        Self {
            id: request_magic!(0x95c1a0edab0944cb, 0xa4e5cb3842f7488a),
            revision: Self::LATEST_REVISION,
            response: Response::none(),
            paging_mode,
            max_mode,
            min_mode,
        }
    }
}

/// A request to get the memory map.
#[repr(C)]
pub struct MemoryMapRequest {
    id: [u64; 4],
    revision: u64,
    response: Response<MemoryMapResponse>,
}

impl MemoryMapRequest {
    pub const LATEST_REVISION: u64 = 0;
    request_boilerplate!(MemoryMapResponse);

    /// Creates a new request.
    pub const fn new() -> Self {
        Self {
            id: request_magic!(0x67cf3d9d378a806f, 0xe304acdfc50c3c62),
            revision: Self::LATEST_REVISION,
            response: Response::none(),
        }
    }
}

/// A request to get the entry point.
#[repr(C)]
pub struct EntryPointRequest {
    id: [u64; 4],
    revision: u64,
    response: Response<EntryPointResponse>,
    entry_point: u64,
}

impl EntryPointRequest {
    pub const LATEST_REVISION: u64 = 0;
    request_boilerplate!(EntryPointResponse);

    /// Creates a new request.
    pub const fn new(entry_point: u64) -> Self {
        Self {
            id: request_magic!(0x13d86c035a1cd3e1, 0x2b0caa89d8f3026a),
            revision: Self::LATEST_REVISION,
            response: Response::none(),
            entry_point,
        }
    }
}

/// A request to get the executable file (kernel).
#[repr(C)]
pub struct ExecutableFileRequest {
    id: [u64; 4],
    revision: u64,
    response: Response<ExecutableFileResponse>,
}

impl ExecutableFileRequest {
    pub const LATEST_REVISION: u64 = 0;
    request_boilerplate!(ExecutableFileResponse);

    /// Creates a new request.
    pub const fn new() -> Self {
        Self {
            id: request_magic!(0xad97e90e83f1ed67, 0x31eb5d1c5ff23b69),
            revision: Self::LATEST_REVISION,
            response: Response::none(),
        }
    }
}

/// A request to get the modules.
#[repr(C)]
pub struct ModuleRequest {
    id: [u64; 4],
    revision: u64,
    response: Response<ModuleResponse>,

    internal_module_count: u64,
    internal_modules: NonNull<NonNull<InternalModule>>,
}

unsafe impl Send for ModuleRequest {}
unsafe impl Sync for ModuleRequest {}

impl ModuleRequest {
    // TODO: Technically internal modules are revision 1, but we don't support revision 1.
    pub const LATEST_REVISION: u64 = 0;
    request_boilerplate!(ModuleResponse);

    /// Creates a new request.
    pub const fn new() -> Self {
        Self {
            id: request_magic!(0x3e7e279702be32af, 0xca1c4f3bd1280cee),
            revision: Self::LATEST_REVISION,
            response: Response::none(),

            internal_module_count: 0,
            internal_modules: NonNull::dangling(),
        }
    }
}

/// A request to get the RSDP Address.
#[repr(C)]
pub struct RsdpRequest {
    id: [u64; 4],
    revision: u64,
    response: Response<RsdpResponse>,
}

impl RsdpRequest {
    pub const LATEST_REVISION: u64 = 0;
    request_boilerplate!(RsdpResponse);

    /// Creates a new request.
    pub const fn new() -> Self {
        Self {
            id: request_magic!(0xc5e77b6b397e7b43, 0x27637845accdcf3c),
            revision: Self::LATEST_REVISION,
            response: Response::none(),
        }
    }
}

/// A request to get the boot time.
#[repr(C)]
pub struct BootTimeRequest {
    id: [u64; 4],
    revision: u64,
    response: Response<BootTimeResponse>,
}

impl BootTimeRequest {
    pub const LATEST_REVISION: u64 = 0;
    request_boilerplate!(BootTimeResponse);

    /// Creates a new request.
    pub const fn new() -> Self {
        Self {
            id: request_magic!(0x502746e184c088aa, 0xfbc5ec83e6327893),
            revision: Self::LATEST_REVISION,
            response: Response::none(),
        }
    }
}

/// A request to get the executable address of the kernel.
#[repr(C)]
pub struct ExecutableAddressRequest {
    id: [u64; 4],
    revision: u64,
    response: Response<ExecutableAddressResponse>,
}

impl ExecutableAddressRequest {
    pub const LATEST_REVISION: u64 = 0;
    request_boilerplate!(ExecutableAddressResponse);

    /// Creates a new request.
    pub const fn new() -> Self {
        Self {
            id: request_magic!(0x71ba76863cc55f63, 0xb2644a48c516a487),
            revision: Self::LATEST_REVISION,
            response: Response::none(),
        }
    }
}

#[repr(C)]
pub struct SmBiosRequest {
    id: [u64; 4],
    revision: u64,
    response: Response<SmBiosResponse>,
}

impl SmBiosRequest {
    pub const LATEST_REVISION: u64 = 0;
    request_boilerplate!(SmBiosResponse);

    /// Creates a new request.
    pub const fn new() -> Self {
        Self {
            id: request_magic!(0x9e9046f11e095391, 0xaa4a520fefbde5ee),
            revision: Self::LATEST_REVISION,
            response: Response::none(),
        }
    }
}

#[repr(C)]
pub struct EfiSystemTableRequest {
    id: [u64; 4],
    revision: u64,
    response: Response<EfiSystemTableResponse>,
}

impl EfiSystemTableRequest {
    pub const LATEST_REVISION: u64 = 0;
    request_boilerplate!(EfiSystemTableResponse);

    /// Creates a new request.
    pub const fn new() -> Self {
        Self {
            id: request_magic!(0x5ceba5163eaaf6d6, 0x0a6981610cf65fcc),
            revision: Self::LATEST_REVISION,
            response: Response::none(),
        }
    }
}

#[repr(C)]
pub struct EfiMemoryMapRequest {
    id: [u64; 4],
    revision: u64,
    response: Response<EfiMemoryMapResponse>,
}

impl EfiMemoryMapRequest {
    pub const LATEST_REVISION: u64 = 0;
    request_boilerplate!(EfiMemoryMapResponse);

    /// Creates a new request.
    pub const fn new() -> Self {
        Self {
            id: request_magic!(0x7df62a431d6872d5, 0xa4fcdfb3e57306c8),
            revision: Self::LATEST_REVISION,
            response: Response::none(),
        }
    }
}

#[repr(C)]
pub struct MultiprocessorRequest {
    id: [u64; 4],
    revision: u64,
    response: Response<MultiprocessorResponse>,
    flags: u64,
}

impl MultiprocessorRequest {
    pub const LATEST_REVISION: u64 = 0;
    request_boilerplate!(MultiprocessorResponse);

    /// Creates a new request.
    pub const fn new(flags: u64) -> Self {
        Self {
            id: request_magic!(0x9e9046f11e095391, 0xaa4a520fefbde5ee),
            revision: Self::LATEST_REVISION,
            response: Response::none(),
            flags,
        }
    }
}

#[cfg(feature = "risc-v")]
pub mod risc_v {
    use crate::response::riscv_v::BspHardIdResponse;

    #[repr(C)]
    pub struct BspHardIdRequest {
        id: [u64; 4],
        revision: u64,
        response: Response<BspHardIdResponse>,
    }

    impl BspHardIdRequest {
        pub const LATEST_REVISION: u64 = 0;
        request_boilerplate!(BspHardIdResponse);

        /// Creates a new request.
        pub const fn new() -> Self {
            Self {
                id: request_magic!(0x1369359f025525f9, 0x2ff2a56178391bb6),
                revision: Self::LATEST_REVISION,
                response: Response::none(),
            }
        }
    }
}
