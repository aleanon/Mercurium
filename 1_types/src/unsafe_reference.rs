use std::ops::{Deref, DerefMut};


/// [UnsafeRef] wrapps an immutable raw pointer to type `T` and implements [Send]
/// so a reference can be sent across threads/async boundaries without reference counting.
/// The user needs to make sure that the value pointed to is not dropped, not moved in memory and not mutated 
/// while the [UnsafeRef] is in use.

pub struct UnsafeRef<T: ?Sized>(*const T);

impl<T: ?Sized> UnsafeRef<T> {
    pub unsafe fn new(value: &T) -> Self {
        Self(value as *const T)
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

/// [UnsafeSlice] wrapps an immutable raw pointer to type `[T]` and implements [Send]
/// so the slice can be sent across threads/async boundaries without reference counting.
/// The user needs to make sure that the value pointed to is not dropped, not moved in memory and not mutated.
#[derive(Clone)]
pub struct UnsafeSlice<T> {
    ptr: *const T,
    len: usize,
}

impl<T> UnsafeSlice<T> {
    pub unsafe fn new(value: &[T]) -> Self {
        Self {
            ptr: value.as_ptr(),
            len: value.len(),
        }
    }
}

impl<T> Deref for UnsafeSlice<T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
    }
}

unsafe impl<T> Send for UnsafeSlice<T> {}

/// [UnsafeRefMut] (Mutable Unsafe Reference) wrapps a mutable raw pointer to type `T` and implements [Send]
/// so a mutable reference can be sent across threads/async boundaries without reference counting.
/// The user needs to make sure that the value pointed to is not dropped, not moved in memory and still only has one mutable reference at any time.
/// Be careful not to compare references returned from this type, use only for reading and writing through the wrapped pointer.
pub struct UnsafeRefMut<T: ?Sized>(*mut T);

impl<T: ?Sized> UnsafeRefMut<T> {
    pub unsafe fn new(value: &mut T) -> Self {
        Self(value as *mut T)
    }
}

impl<T: ?Sized> Deref for UnsafeRefMut<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}

impl<T: ?Sized> DerefMut for UnsafeRefMut<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0 }
    }
}

unsafe impl<T: ?Sized + Send> Send for UnsafeRefMut<T> {}
