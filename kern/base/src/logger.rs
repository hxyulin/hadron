use no_alloc::ringbuf::RingBuf;
use spin::Mutex;

pub struct KernelLogger {
    buf: RingBuf<u8, 1024>,
    fallback: Mutex<Option<()>>,
}

impl KernelLogger {
    /// Creates a new Kernel Logger
    pub const fn new() -> KernelLogger {
        KernelLogger {
            buf: RingBuf::new(),
            fallback: Mutex::new(None),
        }
    }
}

impl log::Log for KernelLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        
    }

    fn log(&self, record: &log::Record) {
        
    }

    fn flush(&self) {
        while !self.buf.is_empty() {
            let c = self.buf.pop().unwrap();

        }
    }
}
