use core::mem::MaybeUninit;

/// A fixed-size array, which has vector-like operations.
#[derive(Debug, Clone)]
pub struct ArrayVec<T: Copy, const N: usize> {
    data: [MaybeUninit<T>; N],
    len: usize,
}

#[derive(Debug, Clone)]
pub enum ArrayVecError {
    CapacityOverflow,
}

impl core::fmt::Display for ArrayVecError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::CapacityOverflow => f.write_str("capacity overflow"),
        }
    }
}

impl core::error::Error for ArrayVecError {}

impl<T, const N: usize> ArrayVec<T, N>
where
    T: Copy,
{
    /// Creates a new `ArrayVec` with no elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use no_alloc::vec::ArrayVec;
    ///
    /// let mut vec = ArrayVec::<u8, 4>::new();
    /// assert_eq!(vec.len(), 0);
    /// assert!(vec.is_empty());
    /// ```
    pub const fn new() -> Self {
        Self {
            data: [MaybeUninit::uninit(); N],
            len: 0,
        }
    }

    /// Tries to push a value into the `ArrayVec`.
    ///
    /// # Errors
    ///
    /// Returns an error if the `ArrayVec` is full.
    ///
    /// # Examples
    ///
    /// ```
    /// use no_alloc::vec::ArrayVec;
    ///
    /// let mut vec = ArrayVec::<u8, 1>::new();
    /// assert!(vec.try_push(1).is_ok());
    /// assert_eq!(vec.len(), 1);
    /// assert!(vec.try_push(2).is_err());
    /// ```
    pub fn try_push(&mut self, value: T) -> Result<(), ArrayVecError> {
        if self.len == N {
            return Err(ArrayVecError::CapacityOverflow);
        }
        self.data[self.len].write(value);
        self.len += 1;
        Ok(())
    }

    /// Pushes a value into the `ArrayVec`.
    ///
    /// # Panics
    ///
    /// Panics if the `ArrayVec` is full.
    pub fn push(&mut self, value: T) {
        self.try_push(value).expect("ArrayVec: ran out of capacity");
    }

    /// Returns the number of elements in the `ArrayVec`.
    ///
    /// # Examples
    ///
    /// ```
    /// use no_alloc::vec::ArrayVec;
    ///
    /// let mut vec = ArrayVec::<u8, 4>::new();
    /// assert_eq!(vec.len(), 0);
    /// vec.push(1);
    /// assert_eq!(vec.len(), 1);
    /// vec.push(2);
    /// assert_eq!(vec.len(), 2);
    /// ```
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns true if the `ArrayVec` is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use no_alloc::vec::ArrayVec;
    ///
    /// let mut vec = ArrayVec::<u8, 4>::new();
    /// assert!(vec.is_empty());
    /// vec.push(1);
    /// assert!(!vec.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns a slice of all the elements in the `ArrayVec`.
    ///
    /// # Examples
    ///
    /// ```
    /// use no_alloc::vec::ArrayVec;
    ///
    /// let mut vec = ArrayVec::<u8, 4>::new();
    /// vec.push(1);
    /// vec.push(2);
    /// vec.push(3);
    /// vec.push(4);
    ///
    /// assert_eq!(vec.as_slice(), &[1, 2, 3, 4]);
    /// ```
    pub fn as_slice(&self) -> &[T] {
        // SAFETY: We push elements into the array, so the index is always valid
        unsafe { core::slice::from_raw_parts(self.data.as_ptr() as *const T, self.len) }
    }

    /// Returns a mutable slice of all the elements in the `ArrayVec`.
    ///
    /// # Examples
    ///
    /// ```
    /// use no_alloc::vec::ArrayVec;
    ///
    /// let mut vec = ArrayVec::<u8, 4>::new();
    /// vec.push(1);
    /// vec.push(2);
    /// vec.push(3);
    /// vec.push(4);
    ///
    /// assert_eq!(vec.as_mut_slice(), &mut [1, 2, 3, 4]);
    /// ```
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        // SAFETY: We push elements into the array, so the index is always valid
        unsafe { core::slice::from_raw_parts_mut(self.data.as_mut_ptr() as *mut T, self.len) }
    }

    /// Returns an iterator over the elements of the `ArrayVec`.
    ///
    /// # Examples
    ///
    /// ```
    /// use no_alloc::vec::ArrayVec;
    ///
    /// let mut vec = ArrayVec::<u8, 4>::new();
    /// vec.push(1);
    /// vec.push(2);
    /// vec.push(3);
    /// vec.push(4);
    ///
    /// for i in vec.iter() {
    ///     assert!(*i == 1 || *i == 2 || *i == 3 || *i == 4);
    ///     // do something
    /// }
    /// ```
    pub fn iter(&self) -> core::slice::Iter<'_, T> {
        self.as_slice().iter()
    }

    /// Returns a mutable iterator over the elements of the `ArrayVec`.
    ///
    /// # Examples
    ///
    /// ```
    /// use no_alloc::vec::ArrayVec;
    ///
    /// let mut vec = ArrayVec::<u8, 4>::new();
    /// vec.push(1);
    /// vec.push(2);
    /// vec.push(3);
    /// vec.push(4);
    ///
    /// for i in vec.iter_mut() {
    ///     *i += 1;
    ///     // do something
    /// }
    /// ```
    pub fn iter_mut(&mut self) -> core::slice::IterMut<'_, T> {
        self.as_mut_slice().iter_mut()
    }

    /// Reverses the order of the elements in the `ArrayVec`.
    ///
    /// # Examples
    ///
    /// ```
    /// use no_alloc::vec::ArrayVec;
    ///
    /// let mut vec = ArrayVec::<u8, 4>::new();
    /// vec.push(1);
    /// vec.push(2);
    /// vec.push(3);
    /// vec.push(4);
    ///
    /// vec.reverse();
    ///
    /// assert_eq!(vec.as_slice(), &[4, 3, 2, 1]);
    /// ```
    pub fn reverse(&mut self) {
        let len = self.len;
        for i in 0..len / 2 {
            let j = len - i - 1;
            self.data.swap(i, j);
        }
    }

    /// Returns a pointer to the underlying data.
    pub fn as_ptr(&self) -> *const T {
        self.data.as_ptr() as *const T
    }

    /// Returns a mutable pointer to the underlying data.
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.data.as_mut_ptr() as *mut T
    }
}

impl<T, const U: usize> ArrayVec<T, U>
where
    T: Copy + PartialEq,
{
    /// Returns true if the `ArrayVec` contains the given value.
    ///
    /// # Examples
    /// ```
    /// use no_alloc::vec::ArrayVec;
    ///
    /// let mut vec = ArrayVec::<u8, 4>::new();
    /// vec.push(1);
    /// vec.push(2);
    /// vec.push(3);
    /// vec.push(4);
    ///
    /// assert!(vec.contains(&1));
    /// assert!(!vec.contains(&5));
    /// ```
    pub fn contains(&self, value: &T) -> bool {
        self.iter().any(|x| x == value)
    }
}

impl<T, const U: usize> ArrayVec<T, U>
where
    T: Copy + Ord,
{
    /// Sorts the `ArrayVec` in place.
    /// The sort is not stable, meaning that the relative order of elements that are equal is not
    /// preserved.
    ///
    /// # Examples
    ///
    /// ```
    /// use no_alloc::vec::ArrayVec;
    ///
    /// let mut vec = ArrayVec::<u8, 4>::new();
    /// vec.push(1);
    /// vec.push(2);
    /// vec.push(3);
    /// vec.push(4);
    ///
    /// vec.sort_unstable();
    ///
    /// assert_eq!(vec.as_slice(), &[1, 2, 3, 4]);
    /// ```
    pub fn sort_unstable(&mut self) {
        self.as_mut_slice().sort_unstable();
    }
}

impl<T, const N: usize> core::ops::Index<usize> for ArrayVec<T, N>
where
    T: Copy,
{
    type Output = T;

    /// Returns a reference to the element at the given index.
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use no_alloc::vec::ArrayVec;
    ///
    /// let mut vec = ArrayVec::<u8, 4>::new();
    /// vec.push(1);
    /// vec.push(2);
    /// vec.push(3);
    /// vec.push(4);
    ///
    /// assert_eq!(vec[0], 1);
    /// assert_eq!(vec[1], 2);
    /// assert_eq!(vec[2], 3);
    /// assert_eq!(vec[3], 4);
    /// ```
    fn index(&self, index: usize) -> &Self::Output {
        // SAFETY: We push elements into the array, so the index is always valid
        unsafe { self.data[index].assume_init_ref() }
    }
}

impl<T, const N: usize> core::ops::IndexMut<usize> for ArrayVec<T, N>
where
    T: Copy,
{
    /// Returns a mutable reference to the element at the given index.
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use no_alloc::vec::ArrayVec;
    ///
    /// let mut vec = ArrayVec::<u8, 4>::new();
    /// vec.push(1);
    /// vec.push(2);
    /// vec.push(3);
    /// vec.push(4);
    ///
    /// assert_eq!(vec[0], 1);
    /// assert_eq!(vec[1], 2);
    /// assert_eq!(vec[2], 3);
    /// assert_eq!(vec[3], 4);
    ///
    /// vec[0] = 5;
    /// assert_eq!(vec[0], 5);
    /// ```
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        // SAFETY: We push elements into the array, so the index is always valid
        unsafe { self.data[index].assume_init_mut() }
    }
}
