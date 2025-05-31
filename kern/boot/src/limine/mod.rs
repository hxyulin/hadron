use kernel_base::logger::{init_log, FallbackLogger, LOGGER};
use log::Log;

mod requests;

pub fn kernel_entry() -> ! {
    init_log();
    LOGGER.set_fallback(FallbackLogger::new());
    if !requests::BASE_REVISION.is_supported() {
        // Base Revision is not supported
        panic!("Limine Base Revision {} is not supported!", requests::BASE_REVISION.revision());
    }
    panic!("Reached End of Kernel");
}

