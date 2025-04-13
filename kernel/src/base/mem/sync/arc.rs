use core::{marker::Unsize, ops::{CoerceUnsized, DispatchFromDyn}};

use alloc::alloc::{Allocator, Global};

/// An Arc-like type, but a wrapper so that we can implement traits for it
#[doc(alias = "alloc::sync::Arc")]
#[repr(transparent)]
#[derive(Debug)]
pub struct Arc<T, A = Global>(alloc::sync::Arc<T, A>)
where
    T: ?Sized,
    A: Allocator;

impl<T, A> Arc<T, A>
where
    A: Allocator,
{
    pub fn new_in(data: T, alloc: A) -> Self {
        Self(alloc::sync::Arc::new_in(data, alloc))
    }
}

impl<T: ?Sized, A: Allocator + Clone> Clone for Arc<T, A> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> Arc<T, Global> {
    pub fn new(data: T) -> Self {
        Self(alloc::sync::Arc::new(data))
    }
}

impl<T, A> core::ops::Deref for Arc<T, A>
where
    T: ?Sized,
    A: Allocator,
{
    type Target = T;

    fn deref(&self) -> &T {
        self.0.deref()
    }
}

impl<T: ?Sized + Unsize<U>, U: ?Sized, A: Allocator> CoerceUnsized<Arc<U, A>> for Arc<T, A> {}
impl<T: ?Sized + Unsize<U>, U: ?Sized> DispatchFromDyn<Arc<U>> for Arc<T> {}
