#[used]
#[unsafe(link_section = ".requests_start_marker")]
static _START_MARKER: limine::StartMarker = limine::StartMarker::new();

#[used]
#[unsafe(link_section = ".requests")]
pub static BASE_REVISION: limine::BaseRevision = limine::BaseRevision::newest();

#[used]
#[unsafe(link_section = ".requests_end_marker")]
static _END_MARKER: limine::EndMarker = limine::EndMarker::new();

