use core::{
    alloc::Allocator,
    any::Any,
    ops::{Index, IndexMut},
};

use alloc::{boxed::Box, vec::Vec};

pub trait AnySendSync: Any + Send + Sync {}

/// A type-safe dynamicly typed vector
pub struct AnyVec<A1: Allocator = alloc::alloc::Global, A2: Allocator = alloc::alloc::Global> {
    vec: Vec<Box<dyn AnySendSync, A2>, A1>,
}

impl<A: Allocator> AnyVec<alloc::alloc::Global, A> {
    pub fn new() -> Self {
        Self { vec: Vec::new() }
    }
}

impl<A: Allocator> AnyVec<A, alloc::alloc::Global> {
    pub fn push<T: AnySendSync>(&mut self, value: T) {
        self.vec.push(Box::new(value));
    }
}

impl<A1: Allocator, A2: Allocator> AnyVec<A1, A2> {
    pub fn new_in(alloc: A1) -> Self {
        Self {
            vec: Vec::new_in(alloc),
        }
    }

    pub fn push_in<T: AnySendSync>(&mut self, value: T, alloc: A2) {
        self.vec.push(Box::new_in(value, alloc));
    }

    pub fn len(&self) -> usize {
        self.vec.len()
    }

    pub fn is_empty(&self) -> bool {
        self.vec.is_empty()
    }
}

impl<A: Allocator> Index<usize> for AnyVec<A> {
    type Output = Box<dyn AnySendSync>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.vec[index]
    }
}

impl<A: Allocator> IndexMut<usize> for AnyVec<A> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.vec[index]
    }
}
