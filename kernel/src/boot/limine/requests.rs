use limine::request::{
    BootloaderInfoRequest, ExecutableAddressRequest, ExecutableFileRequest, FirmwareTypeRequest, FramebufferRequest,
    HhdmRequest, MemoryMapRequest, RequestsEndMarker, RequestsStartMarker, StackSizeRequest,
};

#[used]
#[unsafe(link_section = ".requests_start_marker")]
static _START_MARKER: RequestsStartMarker = RequestsStartMarker::new();

#[used]
#[unsafe(link_section = ".requests")]
pub static BASE_REVISION: limine::BaseRevision = limine::BaseRevision::newest();

#[used]
#[unsafe(link_section = ".requests")]
pub static BOOTLOADER_INFO: BootloaderInfoRequest = BootloaderInfoRequest::new();

#[used]
#[unsafe(link_section = ".requests")]
pub static FIRMWARE_TYPE: FirmwareTypeRequest = FirmwareTypeRequest::new();

#[used]
#[unsafe(link_section = ".requests")]
pub static _STACK_SIZE: StackSizeRequest = StackSizeRequest::new(0x10000);

#[used]
#[unsafe(link_section = ".requests")]
pub static HHDM: HhdmRequest = HhdmRequest::new();

#[used]
#[unsafe(link_section = ".requests")]
pub static FRAMEBUFFER: FramebufferRequest = FramebufferRequest::new();

#[used]
#[unsafe(link_section = ".requests")]
pub static MEMORY_MAP: MemoryMapRequest = MemoryMapRequest::new();

#[used]
#[unsafe(link_section = ".requests")]
pub static EXECUTABLE_FILE: ExecutableFileRequest = ExecutableFileRequest::new();

#[used]
#[unsafe(link_section = ".requests")]
pub static EXECUTABLE_ADDRESS: ExecutableAddressRequest = ExecutableAddressRequest::new();

#[used]
#[unsafe(link_section = ".requests_end_marker")]
static _END_MARKER: RequestsEndMarker = RequestsEndMarker::new();
