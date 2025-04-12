/// A volatile cell to the value of type `T`.
///
/// This can be used as a volatile reference as well as a volatile mutable reference.
#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
pub struct VolatileCell<T: Copy>(T);

impl<T> VolatileCell<T>
where
    T: Copy,
{
    /// Creates a `&VolatileCell<T>` from a reference to a value of type `T`.
    pub fn from_ref(t: &T) -> &Self {
        // SAFETY: The pointer is valid and aligned
        unsafe { &*(t as *const T as *const Self) }
    }

    /// Creates a `&mut VolatileCell<T>` from a mutable reference to a value of type `T`.
    pub fn from_mut(t: &mut T) -> &mut Self {
        // SAFETY: The pointer is valid and aligned
        unsafe { &mut *(t as *mut T as *mut Self) }
    }

    /// Volatile reads the value from the pointer.
    pub fn get(&self) -> T {
        // SAFETY: The pointer is valid and aligned
        unsafe { core::ptr::read_volatile(&self.0) }
    }

    /// Volatile writes the value to the pointer.
    pub fn set(&mut self, value: T) {
        // SAFETY: The pointer is valid and aligned
        unsafe { core::ptr::write_volatile(&mut self.0, value) }
    }

    /// Returns a raw pointer to the value.
    pub fn as_ptr(&self) -> *const T {
        self as *const Self as *const T
    }

    /// Returns a raw mut pointer to the value.
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self as *mut Self as *mut T
    }
}
