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

    pub const fn new() -> Self {
        Self {
            buf: [const { MaybeUninit::uninit() }; N],
            head: 0,
            tail: 0,
        }
    }

    /// Returns true of the ring buffer is empty
    pub const fn is_empty(&self) -> bool {
        // Because we only supported SIZE-1 as maximum capacity,
        // buf is empty when head == tail
        self.head == self.tail
    }

    /// Returns the length of the used buffer
    pub const fn len(&self) -> usize {
        (self.head + Self::SIZE - self.tail) % Self::SIZE
    }

    /// Returns true if the buffer is full
    pub const fn is_full(&self) -> bool {
        (self.head + 1) % N == self.tail
    }

    pub const fn max_capacity(&self) -> usize {
        Self::SIZE - 1
    }

    /// Pushes (or enqueues) an element on the ring buffer
    pub fn push(&mut self, x: T) -> Result<(), T> {
        if self.is_full() {
            return Err(x);
        }

        self.buf[self.head].write(x);
        self.head = (self.head + 1) % N;
        Ok(())
    }

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
        buf.push(15);
        buf.push(42);
        assert_eq!(unsafe { buf.buf[0].assume_init_read() }, 15);
        assert_eq!(unsafe { buf.buf[1].assume_init_read() }, 42);
    }

    #[test]
    fn test_push_rollover() {
        let mut buf = RingBuf::<u8, 1024>::new();
        assert_eq!(buf.max_capacity(), 1023);
        for i in 0..1023 {
            let res = buf.push((i % 255) as u8);
            assert!(res.is_ok());
        }
        assert!(buf.push(1).is_err());
        assert_eq!(buf.len(), 1023);

        // Now we pop one
        let res = buf.pop();
        assert!(res.is_some());
        assert_eq!(res.unwrap(), 0);
        assert_eq!(buf.len(), 1022);
        let res = buf.push(1);
        assert!(res.is_ok());
        assert_eq!(buf.len(), 1023);
        assert!(buf.is_full());
    }
}
