use core::{
    ops::{Deref, DerefMut},
    sync::atomic::AtomicBool,
};

use super::UninitCell;

pub struct UninitMutex<T> {
    lock: AtomicBool,
    data: UninitCell<T>,
}

pub struct UninitMutexGuard<'a, T> {
    lock: &'a AtomicBool,
    data: *mut T,
}

impl<T> UninitMutex<T> {
    pub const fn uninit() -> Self {
        Self {
            lock: AtomicBool::new(false),
            data: UninitCell::uninit(),
        }
    }

    pub fn replace(&self, data: T) -> T {
        assert!(!self.lock.load(core::sync::atomic::Ordering::Relaxed));
        unsafe { self.data.replace(data) }
    }

    pub fn lock(&self) -> UninitMutexGuard<'_, T> {
        self.lock.store(true, core::sync::atomic::Ordering::Relaxed);
        UninitMutexGuard {
            lock: &self.lock,
            data: self.data.get_mut_ptr(),
        }
    }
}

impl<'a, T> Deref for UninitMutexGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.data }
    }
}

impl<'a, T> DerefMut for UninitMutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.data }
    }
}

impl<'a, T> Drop for UninitMutexGuard<'a, T> {
    fn drop(&mut self) {
        self.lock.store(false, core::sync::atomic::Ordering::Relaxed);
    }
}

unsafe impl<'a, T> Sync for UninitMutexGuard<'a, T> {}
unsafe impl<'a, T> Send for UninitMutexGuard<'a, T> {}
