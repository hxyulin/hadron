use core::panic::PanicInfo;

use crate::{
    arch::{instructions::interrupts, x86_64::io::uart::Uart16550},
    sync::cell::RacyCell,
};

mod request;

static SERIAL: RacyCell<Option<Uart16550>> = RacyCell::new(None);

fn debug_write_fmt(args: core::fmt::Arguments) {
    use core::fmt::Write;

    if let Some(serial) = SERIAL.get_mut() {
        serial.write_fmt(args).unwrap();
    }
}

macro_rules! debug_print {
    ($expr:expr) => {
        debug_write_fmt(format_args!(concat!("BOOT: ", $expr)))
    };
    ($expr:expr, $($arg:tt)*) => {
        debug_write_fmt(format_args!(concat!("BOOT: ", $expr), $($arg)*))
    };
}

macro_rules! debug_println {
    () => {
        debug_print!("\n")
    };
    ($expr:expr) => {
        debug_print!(concat!($expr, "\n"))
    };
    ($expr:expr, $($arg:tt)*) => {
        debug_print!(concat!($expr, "\n"), $($arg)*)
    }
}

pub fn entry() -> ! {
    // Register Alternate Panic Handler
    *crate::util::panicking::ALT_PANIC_HANDLER.lock() = Some(panic);
    unsafe {
        interrupts::disable();

        init_serial();
        init_core();
    };

    unsafe extern "Rust" {
        fn kernel_main() -> !;
    }
    unsafe { kernel_main() };
}

fn panic(info: &PanicInfo) -> ! {
    debug_println!("\n--- BOOT PANIC ---");
    debug_println!("message: {}", info);
    debug_println!("\n--- END PANIC ---");
    loop {}
}

unsafe fn init_serial() {
    let mut writer = unsafe { Uart16550::new(0x3F8) };
    unsafe { writer.init() };
    SERIAL.replace(Some(writer));
    debug_println!("Initialized Serial");
}

unsafe fn init_core() {
    // Init GDT
    unsafe {
        crate::arch::x86_64::core::gdt::init();
        crate::arch::x86_64::core::idt::init();
    }
}
