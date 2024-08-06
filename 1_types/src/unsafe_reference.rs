use std::ops::{Deref, DerefMut};

/// `Ur`(Unsafe Reference) wrapps an immutable raw pointer to type `T` and implements `Send`
/// so a reference can be sent across threads/async boundaries without reference counting.
/// The user needs to make sure that the value pointed to is not dropped, not moved in memory and not mutated while the Ur is in use.
#[derive(Clone)]
pub struct Ur<T>(*const T);

impl<T> Ur<T> {
    pub unsafe fn new(value: &T) -> Self {
        let ptr: *const T = value;
        Self(ptr)
    }
}

impl<T> Deref for Ur<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}

unsafe impl<T> Send for Ur<T> {}

pub struct USr<T> {
    ptr: *const T,
    len: usize,
}

impl<T> USr<T> {
    pub unsafe fn new(value: &[T]) -> Self {
        Self {
            ptr: value.as_ptr(),
            len: value.len(),
        }
    }
}

impl<T> Deref for USr<T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
    }
}

unsafe impl<T> Send for USr<T> {}

/// `MutUr`(Mutable Unsafe Reference) wrapps a mutable raw pointer to type `T` and implements `Send`
/// so a mutable reference can be sent across threads/async boundaries without reference counting.
/// The user needs to make sure that the value pointed to is not dropped, not moved in memory and has no colliding reads or writes while the MutUr is in use,
pub struct MutUr<T>(*mut T);

impl<T> MutUr<T> {
    pub unsafe fn new(value: &mut T) -> Self {
        let ptr: *mut T = value;
        Self(&mut *ptr)
    }
}

impl<T> Deref for MutUr<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}

impl<T> DerefMut for MutUr<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0 }
    }
}

unsafe impl<T> Send for MutUr<T> {}
