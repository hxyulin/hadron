use core::marker::Tuple;

use alloc::{boxed::Box, vec::Vec};
use futures::future::BoxFuture;
use spin::Mutex;

use super::{DeviceFunction, DeviceFunctionInner, DeviceId};

pub mod drm;

// ======================
// Driver implementations
// ======================

pub mod qemu;


/// An experimental async function type
pub struct AsyncFn<Args, R>(Box<dyn Fn(Args) -> BoxFuture<'static, R>>);

impl<Args, R> AsyncFn<Args, R> {
    pub fn new<F, Fut>(f: F) -> Self
    where
        F: Fn(Args) -> Fut + 'static + Send,
        Fut: Future<Output = R> + 'static + Send,
    {
        Self(Box::new(move |args: Args| Box::pin(f(args))))
    }

    pub async fn call(&self, args: Args) -> R {
        (self.0)(args).await
    }
}

pub struct DriverVTable {
    pub probe: fn(&DeviceFunctionInner) -> bool,
    /// The function to initialize the device
    ///
    /// Returns true if the device was initialized successfully
    pub init: fn(&DeviceFunctionInner) -> bool,
}

/// A structure representing a built-in driver
pub struct BuiltinDriver {
    /// The name of the driver
    ///
    /// This is used for logging,
    /// and displayed in the device tree
    name: &'static str,
    match_fn: fn(&DeviceId) -> bool,
    vtable_fn: fn() -> DriverVTable,
}

impl BuiltinDriver {
    pub fn load(&self) -> LoadedDriver {
        let vtable = (self.vtable_fn)();
        LoadedDriver {
            name: self.name,
            vtable,
        }
    }

    pub fn matches(&self, device_id: &DeviceId) -> bool {
        (self.match_fn)(device_id)
    }
}

pub static BUILTIN_DRIVERS: &[BuiltinDriver] = &[BuiltinDriver {
    name: "qemu-vga",
    match_fn: qemu::vga::matches,
    vtable_fn: qemu::vga::get_vtable,
}];

pub struct LoadedDriver {
    pub name: &'static str,
    pub vtable: DriverVTable,
}

impl LoadedDriver {
    pub fn probe(&self, device: &DeviceFunctionInner) -> bool {
        (self.vtable.probe)(device)
    }

    pub fn init(&self, device: &DeviceFunctionInner) -> bool {
        (self.vtable.init)(device)
    }
}

pub static LOADED_DRIVERS: Mutex<Vec<LoadedDriver>> = Mutex::new(Vec::new());
