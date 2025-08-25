#![no_std]
#![allow(static_mut_refs)]

pub mod ansi;
mod qemu;
pub use qemu::{ExitCode, exit_qemu};
#[cfg(target_arch = "x86_64")]
mod serial;

#[cfg(target_arch = "x86_64")]
lazy_static::lazy_static! {
    static ref SERIAL_PORT: spin::Mutex<serial::SerialPort> = spin::Mutex::new(unsafe { serial::SerialPort::new(0x3F8) });
}

pub fn write_fmt(args: core::fmt::Arguments) {
    use core::fmt::Write;
    #[cfg(target_arch = "x86_64")]
    SERIAL_PORT.lock().write_fmt(args).unwrap();
}

pub fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("{}[fail]{}", ansi::RED, ansi::RESET);
    println!("{}", info);
    exit_qemu(ExitCode::Failed);
    unreachable!()
}

#[macro_export]
macro_rules! test_entry {
    ($name: ident) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn $name() -> ! {
            #[cfg(test)]
            test_main();
            $crate::exit_qemu($crate::ExitCode::Success);
            panic!("test_entry_inner should not exit");
        }

        #[panic_handler]
        fn panic_handler(info: &core::panic::PanicInfo) -> ! {
            $crate::panic(info);
        }
    };
    ($name: ident, $init_fn: expr) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn $name() -> ! {
            $init_fn();
            #[cfg(test)]
            test_main();
            $crate::exit_qemu($crate::ExitCode::Success);
            panic!("test_entry_inner should not exit");
        }

        #[panic_handler]
        fn panic_handler(info: &core::panic::PanicInfo) -> ! {
            $crate::panic(info);
        }
    };
}

pub trait Testable {
    fn name(&self) -> &str {
        core::any::type_name::<Self>()
    }
    fn run(&self);
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        self()
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    for test in tests {
        print!("{}...\t", test.name());
        test.run();
        println!("{}[ok]{}", ansi::GREEN, ansi::RESET);
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::write_fmt(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! println {
    () => {
        $crate::print!("\n");
    };
    ($fmt:literal) => {
        $crate::print!(concat!($fmt, "\n"));
    };
    ($fmt:literal, $($arg:tt)*) => {
        $crate::print!(concat!($fmt, "\n"), $($arg)*);
    };
}
