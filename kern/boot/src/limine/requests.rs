use limine::{
    BaseRevision,
    request::{BootloaderInfoRequest, RequestsEndMarker, RequestsStartMarker},
};

#[used]
#[unsafe(link_section = ".requests_start_marker")]
pub static START_MARKER: RequestsStartMarker = RequestsStartMarker::new();

#[used]
#[unsafe(link_section = ".requests")]
pub static BASE_REVISION: BaseRevision = BaseRevision::newest();

#[used]
#[unsafe(link_section = ".requests")]
pub static BOOTLOADER_INFO: BootloaderInfoRequest = BootloaderInfoRequest::new();

#[used]
#[unsafe(link_section = ".requests_end_marker")]
pub static END_MARKER: RequestsEndMarker = RequestsEndMarker::new();
