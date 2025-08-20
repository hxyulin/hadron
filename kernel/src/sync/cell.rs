use core::cell::UnsafeCell;

/// An Unsafe Interior Mutable Cell
/// This is only safe in a single-threaded context
pub struct RacyCell<T>(UnsafeCell<T>);

unsafe impl<T> Sync for RacyCell<T> {}
unsafe impl<T> Send for RacyCell<T> {}

impl<T> RacyCell<T> {
    pub const fn new(value: T) -> Self {
        Self(UnsafeCell::new(value))
    }

    pub fn get(&self) -> &T {
        unsafe { &*self.0.get() }
    }

    pub fn get_mut(&self) -> &mut T {
        unsafe { &mut (*self.0.get()) }
    }

    pub fn get_mut_ptr(&self) -> *mut T {
        self.0.get() as *mut T
    }

    pub fn replace(&self, value: T) -> T {
        unsafe { self.0.replace(value) }
    }
}
