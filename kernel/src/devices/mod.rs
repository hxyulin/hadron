use core::{any::Any, sync::atomic::AtomicUsize};

use alloc::{collections::btree_map::BTreeMap, sync::Arc};
use spin::{Mutex, RwLock};

pub mod fb;
pub mod tty;

pub struct DeviceManager {
    tty: DeviceList<dyn tty::TtyDevice>,
    fb: DeviceList<dyn fb::FbDevice>,
}

impl DeviceManager {
    pub fn new() -> Self {
        Self {
            tty: DeviceList::new(),
            fb: DeviceList::new(),
        }
    }

    pub fn add_tty_device(&self, device: Arc<Mutex<dyn tty::TtyDevice>>) -> DeviceId {
        self.tty.add_device(device)
    }

    pub fn add_fb_device(&self, device: Arc<Mutex<dyn fb::FbDevice>>) -> DeviceId {
        self.fb.add_device(device)
    }

    pub fn get_tty_device(&self, id: DeviceId) -> Option<Arc<Mutex<dyn tty::TtyDevice>>> {
        self.tty.devices.read().get(&id).map(|device| device.clone())
    }

    pub fn get_fb_device(&self, id: DeviceId) -> Option<Arc<Mutex<dyn fb::FbDevice>>> {
        self.fb.devices.read().get(&id).map(|device| device.clone())
    }
}

pub type DeviceId = usize;

pub struct DeviceList<T: ?Sized> {
    id_gen: AtomicUsize,
    devices: RwLock<BTreeMap<DeviceId, Arc<Mutex<T>>>>,
}

impl<T> DeviceList<T>
where
    T: ?Sized,
{
    pub fn new() -> Self {
        Self {
            id_gen: AtomicUsize::new(0),
            devices: RwLock::new(BTreeMap::new()),
        }
    }

    pub fn add_device(&self, device: Arc<Mutex<T>>) -> DeviceId {
        let id = self.id_gen.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
        self.devices.write().insert(id, device);
        id
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceClass {
    Tty = 0,
    Framebuffer = 1,
}

/// A base trait for all devices.
pub trait Device {
    fn class_id(&self) -> DeviceClass;
    fn as_any(&self) -> &dyn Any;
}
