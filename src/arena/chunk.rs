use std::{mem::MaybeUninit, ptr::NonNull};

pub(crate) struct Chunk<T = u8> {
    /// The raw storage for the arena chunk.
    pub(crate) storage: NonNull<[MaybeUninit<T>]>,
    /// The number of valid entries in the chunk.
    entries: usize,
}

impl<T> Drop for Chunk<T> {
    fn drop(&mut self) {
        unsafe { drop(Box::from_raw(self.storage.as_mut())) }
    }
}

impl<T> Chunk<T> {
    pub fn new(cap: usize) -> Self {
        Chunk {
            storage: NonNull::from(Box::leak(Box::new_uninit_slice(cap))),
            entries: 0,
        }
    }

    pub fn start(&mut self) -> *mut T {
        self.storage.as_ptr() as *mut T
    }

    pub fn end(&mut self) -> *mut T {
        unsafe { self.start().add(self.storage.len()) }
    }
}
