use std::ops::{Deref, DerefMut};

/// [Ur](Unsafe Reference) wrapps an immutable raw pointer to type `T` and implements [Send]
/// so a reference can be sent across threads/async boundaries without reference counting.
/// The user needs to make sure that the value pointed to is not dropped, not moved in memory and not mutated 
/// while the [UnsafeRef] is in use.

pub struct UnsafeRef<T: ?Sized>(*const T);

impl<T: ?Sized> UnsafeRef<T> {
    pub unsafe fn new(value: &T) -> Self {
        let ptr: *const T = value;
        Self(ptr)
    }
}

impl<T: ?Sized> Deref for UnsafeRef<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}

impl<T: ?Sized> Clone for UnsafeRef<T> {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl<T: ?Sized> Copy for UnsafeRef<T>{}

unsafe impl<T: ?Sized> Send for UnsafeRef<T> {}

/// [Us] (Unsafe Slice) wrapps an immutable raw pointer to type `[T]` and implements [Send]
/// so the slice can be sent across threads/async boundaries without reference counting.
/// The user needs to make sure that the value pointed to is not dropped, not moved in memory and not mutated.
#[derive(Clone)]
pub struct Us<T> {
    ptr: *const T,
    len: usize,
}

impl<T> Us<T> {
    pub unsafe fn new(value: &[T]) -> Self {
        Self {
            ptr: value.as_ptr(),
            len: value.len(),
        }
    }
}

impl<T> Deref for Us<T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
    }
}

unsafe impl<T> Send for Us<T> {}

/// [MutUr] (Mutable Unsafe Reference) wrapps a mutable raw pointer to type `T` and implements [Send]
/// so a mutable reference can be sent across threads/async boundaries without reference counting.
/// The user needs to make sure that the value pointed to is not dropped, not moved in memory and has no colliding reads/writes while the MutUr is in use,
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
