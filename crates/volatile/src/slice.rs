use core::{
    intrinsics::{volatile_copy_memory, volatile_copy_nonoverlapping_memory, volatile_set_memory},
    ops::{Bound, Index, IndexMut, RangeBounds},
};

use crate::cell::VolatileCell;

/// A wrapper around a slice that is volatile.
/// This implement slice functions that can be used to do volatile operations on the slice.
#[repr(transparent)]
#[derive(Debug)]
pub struct VolatileSlice<T>([T]);

impl<T> VolatileSlice<T> {
    /// Creates a `&VolatileSlice<T>` from a reference to a slice of type `T`.
    pub fn from_slice(slice: &[T]) -> &Self {
        // SAFETY: The pointer is valid and aligned, and the length is within
        // Technically transmuting directly is also valid, because of the repr(transparent)
        unsafe { &*(slice as *const [T] as *const Self) }
    }

    /// Creates a `&mut VolatileSlice<T>` from a mutable reference to a slice of type `T`.
    pub fn from_slice_mut(slice: &mut [T]) -> &mut Self {
        // SAFETY: The pointer is valid and aligned, and the length is within
        // Technically transmuting directly is also valid, because of the repr(transparent)
        unsafe { &mut *(slice as *mut [T] as *mut Self) }
    }

    /// Returns the length of the slice, which is the number of elements in the slice.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if the slice is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Copies the elements from `other` into `self`
    ///
    /// # Panics
    ///
    /// Panics if `other.len()` is not equal to `self.len()`.
    pub fn copy_from_slice(&mut self, other: &[T]) {
        assert!(other.len() == self.len());
        // SAFETY: The pointer is valid and aligned, and the length is within
        unsafe {
            volatile_copy_nonoverlapping_memory(self.as_mut_ptr(), other.as_ptr(), self.len());
        }
    }

    /// Copies the elements from `self` into `other`
    ///
    /// # Panics
    ///
    /// Panics if `other.len()` is not equal to `self.len()`.
    pub fn copy_to_slice(&self, other: &mut [T]) {
        assert!(other.len() == self.len());
        // SAFETY: The pointer is valid and aligned, and the length is within
        unsafe {
            volatile_copy_nonoverlapping_memory(other.as_mut_ptr(), self.as_ptr(), self.len());
        }
    }

    /// Copies elements within the slice, from `src` to `dst`.
    ///
    /// # Panics
    ///
    /// Panics if `src` is out of bounds.
    /// Panics if start is greater than end.
    /// Panics if end is greater than the length of the slice.
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

    /// Returns a raw pointer to the value.
    pub fn as_ptr(&self) -> *const T {
        self.0.as_ptr()
    }

    /// Returns a raw mut pointer to the value.
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.0.as_mut_ptr()
    }
}

impl VolatileSlice<u8> {
    /// Fills the slice with the given value.
    ///
    /// This is only implemented for types that are byte-sized, because it uses the `volatile_set_memory` intrinsic.
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
