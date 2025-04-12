use core::ptr::NonNull;

/// A volatile non-null pointer to a value of type `T`.
///
/// This type is used to read and write values to memory without causing the compiler to optimize
/// away the reads and writes.
#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
pub struct VolatilePtr<T> {
    ptr: NonNull<T>,
}

impl<T> VolatilePtr<T> {
    /// Creates a new `VolatilePtr` from a raw pointer.
    ///
    /// # Examples
    /// ```
    /// use volatile::ptr::VolatilePtr;
    ///
    /// let mut x = 0;
    /// let ptr = VolatilePtr::new(&mut x);
    /// assert_eq!(ptr.get(), 0);
    /// ptr.set(1);
    /// assert_eq!(ptr.get(), 1);
    /// assert_eq!(x, 1);
    /// ```
    pub fn new(ptr: *mut T) -> Self {
        Self {
            ptr: NonNull::new(ptr).expect("ptr must be non-null"),
        }
    }

    /// Creates a new `VolatilePtr` from a raw pointer without checking for null.
    ///
    /// # Safety
    /// The caller must ensure that the pointer is non-null.
    pub const unsafe fn new_unchecked(ptr: *mut T) -> Self {
        Self {
            ptr: unsafe { NonNull::new_unchecked(ptr) },
        }
    }

    /// Returns a raw pointer to the value.
    /// This pointer shouldnt be used directly, but rather through the `VolatilePtr` API,
    /// or using the `core::ptr::read_volatile` and `core::ptr::write_volatile` functions.
    pub fn as_ptr(&self) -> *mut T {
        self.ptr.as_ptr()
    }

    /// Volatile reads the value from the pointer.
    pub fn get(&self) -> T {
        unsafe { self.ptr.read_volatile() }
    }

    /// Volatile writes the value to the pointer.
    pub fn set(&self, value: T) {
        unsafe { self.ptr.write_volatile(value) }
    }
}
