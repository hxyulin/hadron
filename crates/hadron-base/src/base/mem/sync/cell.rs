use core::{cell::UnsafeCell, mem::MaybeUninit};

/// A 'safe' cell, which allows for interior mutability
///
/// But there are some restrictions:
/// - The cell can only be accessed from a single thread
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

/// A `safe` cell that can be uninitialized
pub struct UninitCell<T>(RacyCell<MaybeUninit<T>>);

impl<T> UninitCell<T> {
    pub const fn uninit() -> Self {
        Self(RacyCell::new(MaybeUninit::uninit()))
    }

    pub const fn new(value: T) -> Self {
        Self(RacyCell::new(MaybeUninit::new(value)))
    }

    pub fn is_initialized(&self) -> bool {
        !self.0.get_mut_ptr().is_null()
    }

    pub unsafe fn get(&self) -> &T {
        debug_assert!(self.is_initialized(), "UninitCell::get is called on an uninitialized cell");
        unsafe { self.0.get().assume_init_ref() }
    }

    /// Replaces the value of the cell with the given value
    /// For replacement of cells that are initialized, use [`UninitCell::replace`]
    ///
    /// # Safety
    /// - The cell must be uninitialized, because the destructor of the value won't be called
    pub unsafe fn replace_uninit(&self, value: T)  {
        core::mem::forget(self.0.replace(MaybeUninit::new(value)));
    }

    /// Replaces the value of the cell with the given value
    /// For replacement of cells that are uninitialized, use [`UninitCell::replace_uninit`]
    ///
    /// # Safety
    /// - The cell must be initialized
    pub unsafe fn replace(&self, value: T) -> T {
        let mut val = self.0.replace(MaybeUninit::new(value));
        assert!(!val.as_mut_ptr().is_null(), "UninitCell::replace is called on an uninitialized cell");
        unsafe { val.assume_init() }
    }

    pub unsafe fn get_mut(&self) -> &mut T {
        debug_assert!(self.is_initialized(), "UninitCell::get_mut is called on an uninitialized cell");
        unsafe { self.0.get_mut().assume_init_mut() }
    }

    pub fn get_mut_ptr(&self) -> *mut T {
        self.0.get_mut().as_mut_ptr() as *mut T
    }
}
