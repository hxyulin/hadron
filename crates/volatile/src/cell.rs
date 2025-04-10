#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
pub struct VolatileCell<T: Copy>(T);

impl<T> VolatileCell<T>
where
    T: Copy,
{
    pub fn from_ref(t: &T) -> &Self {
        // SAFETY: The pointer is valid and aligned
        unsafe { &*(t as *const T as *const Self) }
    }

    pub fn from_mut(t: &mut T) -> &mut Self {
        // SAFETY: The pointer is valid and aligned
        unsafe { &mut *(t as *mut T as *mut Self) }
    }

    pub fn get(&self) -> T {
        // SAFETY: The pointer is valid and aligned
        unsafe { core::ptr::read_volatile(&self.0) }
    }

    pub fn set(&mut self, value: T) {
        // SAFETY: The pointer is valid and aligned
        unsafe { core::ptr::write_volatile(&mut self.0, value) }
    }

    pub fn as_ptr(&self) -> *const T {
        self as *const Self as *const T
    }

    pub fn as_mut_ptr(&mut self) -> *mut T {
        self as *mut Self as *mut T
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;

    #[test]
    fn test_volatile_ref() {
        let mut value = 0;
        let vref = VolatileCell::from_mut(&mut value);
        assert_eq!(vref.get(), 0);
        vref.set(1);
        assert_eq!(value, 1);
    }
}
