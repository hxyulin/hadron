use core::mem::MaybeUninit;

/// A Circular / Ring buffer
/// 'Full' is considered when it is SIZE-1
#[derive(Clone, Copy)]
pub struct RingBuf<T: core::marker::Copy, const SIZE: usize> {
    buf: [MaybeUninit<T>; SIZE],
    head: usize,
    tail: usize,
}

impl<T, const N: usize> RingBuf<T, N>
where
    T: core::marker::Copy,
{
    pub const SIZE: usize = N;

    /// Create a new ringbuf with no data inside
    ///
    /// This method does not allocate memory.
    ///
    /// # Example
    /// ```
    /// use no_alloc::ringbuf::RingBuf;
    ///
    /// // Create an empty ringbuf
    /// let ringbuf = RingBuf::<u8, 8>::new();
    /// assert!(ringbuf.is_empty());
    /// ```
    pub const fn new() -> Self {
        Self {
            buf: [const { MaybeUninit::uninit() }; N],
            head: 0,
            tail: 0,
        }
    }

    /// Returns true of the ring buffer is empty
    ///
    /// # Example
    /// ```
    /// use no_alloc::ringbuf::RingBuf;
    ///
    /// // Create an empty ringbuf
    /// let mut ringbuf = RingBuf::<u8, 8>::new();
    /// assert!(ringbuf.is_empty());
    /// ringbuf.push(42);
    /// assert!(!ringbuf.is_empty());
    /// ````
    pub const fn is_empty(&self) -> bool {
        // Because we only supported SIZE-1 as maximum capacity,
        // buf is empty when head == tail
        self.head == self.tail
    }

    /// Returns the length of the used buffer
    ///
    /// # Example
    /// ```
    /// use no_alloc::ringbuf::RingBuf;
    ///
    /// // Create an empty ringbuf
    /// let mut ringbuf = RingBuf::<u8, 8>::new();
    /// assert_eq!(ringbuf.len(), 0);
    /// ringbuf.push(1);
    /// ringbuf.push(2);
    /// ringbuf.push(3);
    /// assert_eq!(ringbuf.len(), 3);
    /// # let popped =
    /// ringbuf.pop();
    /// # assert_eq!(popped, Some(1));
    /// assert_eq!(ringbuf.len(), 2);
    /// ```
    pub const fn len(&self) -> usize {
        (self.head + Self::SIZE - self.tail) % Self::SIZE
    }

    /// Returns true if the buffer is full
    ///
    /// # Example
    /// ```
    /// use no_alloc::ringbuf::RingBuf;
    ///
    /// // Create an empty ringbuf
    /// let mut ringbuf = RingBuf::<u8, 8>::new();
    /// for i in 0..6 {
    ///     ringbuf.push(i);
    /// }
    /// assert!(!ringbuf.is_full());
    /// ringbuf.push(6);
    /// assert!(ringbuf.is_full());
    /// ````
    pub const fn is_full(&self) -> bool {
        (self.head + 1) % N == self.tail
    }

    /// Returns the maximum number of elements that can be stored in the ringbuf
    /// This is calculated as the size subtracted 1
    pub const fn max_capacity(&self) -> usize {
        Self::SIZE - 1
    }

    /// Pushes (or enqueues) an element on the ring buffer
    ///
    /// # Errors
    /// Returns an error with the pushed value if the ringbuf is full
    pub fn try_push(&mut self, x: T) -> Result<(), T> {
        if self.is_full() {
            return Err(x);
        }

        // SAFETY: We checked that it isn't full
        unsafe { self.push_unchecked(x) };
        Ok(())
    }

    /// Pushes (or enqueues) an element on the ring buffer
    ///
    /// # Panics
    /// Panics if the ringbuf is full. If this is not what you want, see [RingBuf::try_push] or
    /// [RingBuf::push_unchecked]
    pub fn push(&mut self, x: T) {
        if let Err(_) = self.try_push(x) {
            panic!("ringbuf is full");
        }
    }

    /// Pushes (or enqueues) an element on the ring buffer
    ///
    /// # Safety
    /// This does not check if it is out of bounds, which may cause data to be overwritten
    pub unsafe fn push_unchecked(&mut self, x: T) {
        self.buf[self.head].write(x);
        self.head = (self.head + 1) % N;
    }

    /// Pops (or dequeues) an element off the ring buffer
    ///
    /// Returns none if the ringbuf is empty
    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        let x = unsafe { self.buf[self.tail].assume_init_read() };
        self.buf[self.tail] = MaybeUninit::uninit();
        self.tail = (self.tail + 1) % N;
        Some(x)
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;

    #[test]
    fn test_is_empty() {
        let buf = RingBuf::<u8, 1024>::new();
        assert!(buf.is_empty());
    }

    #[test]
    fn test_push() {
        let mut buf = RingBuf::<u8, 1024>::new();
        buf.try_push(15).unwrap();
        buf.try_push(42).unwrap();
        assert_eq!(unsafe { buf.buf[0].assume_init_read() }, 15);
        assert_eq!(unsafe { buf.buf[1].assume_init_read() }, 42);
    }

    #[test]
    fn test_push_rollover() {
        let mut buf = RingBuf::<u8, 1024>::new();
        assert_eq!(buf.max_capacity(), 1023);
        for i in 0..1023 {
            let res = buf.try_push((i % 255) as u8);
            assert!(res.is_ok());
        }
        assert!(buf.try_push(1).is_err());
        assert_eq!(buf.len(), 1023);

        // Now we pop one
        let res = buf.pop();
        assert!(res.is_some());
        assert_eq!(res.unwrap(), 0);
        assert_eq!(buf.len(), 1022);
        let res = buf.try_push(1);
        assert!(res.is_ok());
        assert_eq!(buf.len(), 1023);
        assert!(buf.is_full());
    }
}
