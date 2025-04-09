use limine::request::{
    BootloaderInfoRequest, ExecutableAddressRequest, ExecutableFileRequest, FirmwareTypeRequest, FramebufferRequest,
    HhdmRequest, MemoryMapRequest, RequestsEndMarker, RequestsStartMarker, StackSizeRequest,
};

use crate::serial::SerialPort;

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

#[unsafe(no_mangle)]
pub extern "C" fn kernel_entry() -> ! {
    use core::fmt::Write;
    let mut serial = unsafe { SerialPort::new(0x3F8) };
    if !BASE_REVISION.is_supported() {
        panic!("Limine is not supported");
    }
    let response = BOOTLOADER_INFO.get_response().unwrap();
    writeln!(serial, "Booted from {} {}", response.name(), response.version()).unwrap();
    let response = FIRMWARE_TYPE.get_response().unwrap();
    writeln!(serial, "Firmware type: {:?}", response.firmware_type()).unwrap();
    let hhdm = HHDM.get_response().unwrap();
    writeln!(serial, "HHDM offset: {:#x}", hhdm.offset).unwrap();
    let framebuffer = FRAMEBUFFER.get_response().unwrap();
    for framebuffer in framebuffer.framebuffers() {
        writeln!(serial, "Framebuffer: {:#x?}", framebuffer).unwrap();
    }
    let memory_map = MEMORY_MAP.get_response().unwrap();
    for memory_map_entry in memory_map.memory_map() {
        writeln!(serial, "Memory map: {:#x?}", memory_map_entry).unwrap();
    }

    let file = EXECUTABLE_FILE.get_response().unwrap().executable_file();
    writeln!(serial, "Executable file: {}", file.path()).unwrap();
    writeln!(serial, "Cmdline: {}", file.cmdline()).unwrap();

    let response = EXECUTABLE_ADDRESS.get_response().unwrap();
    writeln!(
        serial,
        "Executable address: (Physical: {:#x}, Virtual: {:#x})",
        response.physical_address, response.virtual_address
    )
    .unwrap();

    panic!("Kernel entry point reached");
}
