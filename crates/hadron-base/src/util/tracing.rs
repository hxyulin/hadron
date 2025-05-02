use core::sync::atomic::AtomicU64;

use spin::Mutex;
use tracing::{level_filters::LevelFilter, span};
use uart_16550::SerialPort;

pub struct SerialSubscriber {
    filter: LevelFilter,
    uart: Mutex<SerialPort>,
}

impl SerialSubscriber {
    /// Creates a new `SerialSubscriber`.
    /// The `filter` is applied to the `LevelFilter` of the `log` crate.
    /// The uart should be initialized before calling this function.
    pub fn new(uart: Mutex<SerialPort>, filter: LevelFilter) -> Self {
        Self { filter, uart }
    }
}

impl tracing::Subscriber for SerialSubscriber {
    fn enabled(&self, metadata: &tracing::Metadata<'_>) -> bool {
        metadata.level() <= &self.filter
    }

    fn register_callsite(&self, _metadata: &'static tracing::Metadata<'static>) -> tracing::subscriber::Interest {
        tracing::subscriber::Interest::always()
    }

    fn new_span(&self, _span: &tracing::span::Attributes<'_>) -> tracing::Id {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        tracing::Id::from_u64(COUNTER.fetch_add(1, core::sync::atomic::Ordering::Relaxed))
    }

    fn record(&self, _span: &span::Id, values: &span::Record<'_>) {
        use core::fmt::Write;
        let mut uart = self.uart.lock();
        uart.write_fmt(format_args!("{:?}\n", values)).unwrap();
    }

    fn record_follows_from(&self, _span: &tracing::Id, _follows: &tracing::Id) {
        // TODO: Implement this
    }

    fn event(&self, event: &tracing::Event<'_>) {
        use core::fmt::Write;
        let mut uart = self.uart.lock();
        uart.write_fmt(format_args!("{:?}\n", event)).unwrap();
    }

    fn enter(&self, span: &tracing::Id) {
        use core::fmt::Write;
        let mut uart = self.uart.lock();
        uart.write_fmt(format_args!("{:?}\n", span)).unwrap();
    }

    fn exit(&self, _span: &tracing::Id) {}
}

pub fn init_tracing() {
    let mut serial = unsafe { SerialPort::new(0x3F8) };
    serial.init();
    tracing::subscriber::set_global_default(SerialSubscriber::new(Mutex::new(serial), LevelFilter::TRACE)).unwrap();
}
