use core::{
    intrinsics::{volatile_copy_memory, volatile_copy_nonoverlapping_memory, volatile_set_memory},
    ops::{Bound, Index, IndexMut, RangeBounds},
};

use crate::cell::VolatileCell;

/// A wrapper around a slice that is volatile.
#[repr(transparent)]
#[derive(Debug)]
pub struct VolatileSlice<T>([T]);

impl<T> VolatileSlice<T> {
    pub fn from_slice(slice: &[T]) -> &Self {
        // SAFETY: The pointer is valid and aligned, and the length is within
        // Technically transmuting directly is also valid, because of the repr(transparent)
        unsafe { &*(slice as *const [T] as *const Self) }
    }

    pub fn from_slice_mut(slice: &mut [T]) -> &mut Self {
        // SAFETY: The pointer is valid and aligned, and the length is within
        // Technically transmuting directly is also valid, because of the repr(transparent)
        unsafe { &mut *(slice as *mut [T] as *mut Self) }
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn copy_from_slice(&mut self, other: &[T]) {
        assert!(other.len() <= self.len());
        // SAFETY: The pointer is valid and aligned, and the length is within
        unsafe {
            volatile_copy_nonoverlapping_memory(self.as_mut_ptr(), other.as_ptr(), other.len());
        }
    }

    pub fn copy_to_slice(&self, other: &mut [T]) {
        assert!(other.len() <= self.len());
        // SAFETY: The pointer is valid and aligned, and the length is within
        unsafe {
            volatile_copy_nonoverlapping_memory(other.as_mut_ptr(), self.as_ptr(), other.len());
        }
    }

    pub fn copy_within<R: RangeBounds<usize>>(&mut self, src: R, dst: usize) {
        let start = match src.start_bound() {
            Bound::Included(&start) => start,
            Bound::Excluded(&start) => start + 1,
            Bound::Unbounded => 0,
        };
        let end = match src.end_bound() {
            Bound::Included(&end) => end + 1,
            Bound::Excluded(&end) => end,
            Bound::Unbounded => self.len(),
        };
        assert!(start <= end);
        assert!(end <= self.len());
        // SAFETY: The pointer is valid and aligned, and the length is within
        unsafe {
            volatile_copy_memory(self.as_mut_ptr().add(dst), self.as_ptr().add(start), end - start);
        }
    }

    pub fn as_ptr(&self) -> *const T {
        self.0.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.0.as_mut_ptr()
    }
}

impl VolatileSlice<u8> {
    pub fn fill(&mut self, value: u8) {
        unsafe { volatile_set_memory(self.as_mut_ptr(), value, self.len()) }
    }
}

/// A trait that is not `usize`.
/// This is bit of a hack to make the `Index` impls work.
trait NotUsize {}
impl<T> NotUsize for core::ops::Range<T> {}
impl<T> NotUsize for core::ops::RangeInclusive<T> {}
impl<T> NotUsize for core::ops::RangeFrom<T> {}
impl<T> NotUsize for core::ops::RangeToInclusive<T> {}
impl<T> NotUsize for core::ops::RangeTo<T> {}
impl NotUsize for core::ops::RangeFull {}

impl<T> Index<usize> for VolatileSlice<T>
where
    T: Copy,
{
    type Output = VolatileCell<T>;

    fn index(&self, index: usize) -> &Self::Output {
        VolatileCell::from_ref(&self.0[index])
    }
}

impl<T> IndexMut<usize> for VolatileSlice<T>
where
    T: Copy,
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        VolatileCell::from_mut(&mut self.0[index])
    }
}

impl<T, R> Index<R> for VolatileSlice<T>
where
    T: Copy,
    R: RangeBounds<usize> + NotUsize,
{
    type Output = VolatileSlice<T>;

    fn index(&self, index: R) -> &Self::Output {
        let start = match index.start_bound() {
            Bound::Included(&start) => start,
            Bound::Excluded(&start) => start + 1,
            Bound::Unbounded => 0,
        };
        let end = match index.end_bound() {
            Bound::Included(&end) => end + 1,
            Bound::Excluded(&end) => end,
            Bound::Unbounded => self.len(),
        };
        assert!(start <= end);
        assert!(end <= self.len());
        VolatileSlice::from_slice(&self.0[start..end])
    }
}

impl<T, R> IndexMut<R> for VolatileSlice<T>
where
    T: Copy,
    R: RangeBounds<usize> + NotUsize,
{
    fn index_mut(&mut self, index: R) -> &mut Self::Output {
        let start = match index.start_bound() {
            Bound::Included(&start) => start,
            Bound::Excluded(&start) => start + 1,
            Bound::Unbounded => 0,
        };
        let end = match index.end_bound() {
            Bound::Included(&end) => end + 1,
            Bound::Excluded(&end) => end,
            Bound::Unbounded => self.len(),
        };
        assert!(start <= end);
        assert!(end <= self.len());
        VolatileSlice::from_slice_mut(&mut self.0[start..end])
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;

    #[test]
    fn test_volatile_slice() {
        let mut slice = [0u8; 10];
        let vslice = VolatileSlice::from_slice_mut(&mut slice);
        assert_eq!(vslice.len(), 10);
        assert_eq!(vslice.as_ptr() as usize, slice.as_ptr() as usize);
    }

    #[test]
    fn test_volatile_slice_index() {
        let mut slice = [0u8; 10];
        let vslice = VolatileSlice::from_slice_mut(&mut slice);
        assert_eq!(vslice[0].get(), 0);
        assert_eq!(vslice[1].get(), 0);
        vslice[0].set(1);
        vslice[1].set(2);
        assert_eq!(slice[0], 1);
        assert_eq!(slice[1], 2);
    }

    #[test]
    fn test_volatile_slice_index_range() {
        let mut slice = [0u8; 10];
        let vslice = VolatileSlice::from_slice_mut(&mut slice);
        assert_eq!(vslice[0..2].len(), 2);
        assert_eq!(vslice[0..2][0].get(), 0);
        assert_eq!(vslice[0..2][1].get(), 0);
        vslice[0..2][0].set(1);
        vslice[0..2][1].set(2);
        assert_eq!(slice[0..2], [1, 2]);
    }

    #[test]
    fn test_copy_to_slice() {
        let mut slice = [1u8, 2u8, 3u8, 4u8];
        let vslice = VolatileSlice::from_slice_mut(&mut slice);
        let mut result = [0u8; 2];
        vslice[..2].copy_to_slice(&mut result);
        assert_eq!(result, [1, 2]);
        vslice[2..4].copy_to_slice(&mut result);
        assert_eq!(result, [3, 4]);
    }
}
