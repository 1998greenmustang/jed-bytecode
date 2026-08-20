use std::{
    alloc::{self, Layout},
    marker::PhantomData,
    mem::ManuallyDrop,
    ptr::{self, NonNull},
};

use crate::error::ProgramErrorKind;

#[derive(Debug)]
pub struct Stack<T> {
    ptr: NonNull<T>,
    cap: usize,
    len: usize,
}

impl<T> Default for Stack<T> {
    fn default() -> Self {
        Self {
            ptr: NonNull::dangling(),
            cap: Default::default(),
            len: Default::default(),
        }
    }
}

unsafe impl<T: Send> Send for Stack<T> {}

unsafe impl<T: Sync> Sync for Stack<T> {}

impl<T> Stack<T> {
    pub fn new() -> Stack<T> {
        let mut this: Stack<T> = Stack {
            ptr: NonNull::dangling(),
            len: 0,
            cap: 0,
        };
        this.grow();
        this
    }

    fn grow(&mut self) {
        let (new_cap, new_layout) = if self.cap == 0 {
            (1, Layout::array::<T>(1).unwrap())
        } else {
            let new_cap = 2 * self.cap;
            let new_layout = Layout::array::<T>(new_cap).unwrap();
            (new_cap, new_layout)
        };

        assert!(new_layout.size() <= isize::MAX as usize, "too big");

        let new_ptr = if self.cap == 0 {
            unsafe { alloc::alloc(new_layout) }
        } else {
            let old_layout = Layout::array::<T>(self.cap).unwrap();
            let old_ptr = self.ptr.as_ptr() as *mut u8;
            unsafe { alloc::realloc(old_ptr, old_layout, new_layout.size()) }
        };

        self.ptr = match NonNull::new(new_ptr as *mut T) {
            Some(p) => p,
            None => alloc::handle_alloc_error(new_layout),
        };
        self.cap = new_cap;
    }

    pub fn push(&mut self, elem: T) {
        if self.len == self.cap {
            self.grow();
        }
        unsafe {
            ptr::write(self.ptr.as_ptr().add(self.len), elem);
        }
        self.len += 1;
    }

    pub fn pop(&mut self) -> Result<T, ProgramErrorKind> {
        if self.len == 0 {
            return Err(ProgramErrorKind::StackError(1));
        }
        self.len -= 1;
        Ok(unsafe { ptr::read(self.ptr.as_ptr().add(self.len)) })
    }

    pub fn pop_if(&mut self, predicate: impl FnOnce(&mut T) -> bool) -> Option<T> {
        let last = self.last_mut_option()?;
        if predicate(last) {
            Some(unsafe { ptr::read(self.ptr.as_ptr().add(self.len)) })
        } else {
            None
        }
    }

    pub fn pop_mut(&mut self) -> Result<&mut T, ProgramErrorKind> {
        if self.len == 0 {
            return Err(ProgramErrorKind::StackError(1));
        }
        self.len -= 1;
        Ok(unsafe { &mut *self.ptr.as_ptr().add(self.len) })
    }

    pub fn pop_n(&mut self, n: usize) -> Result<&[T], ProgramErrorKind> {
        unsafe {
            if n > self.len {
                Err(ProgramErrorKind::StackError(n))
            } else {
                let nth = &*self.ptr.as_ptr().add(self.len - n);
                self.len -= n;
                Ok(std::slice::from_raw_parts(nth, n))
            }
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn last(&self) -> Result<&T, ProgramErrorKind> {
        if self.len == 0 {
            Err(ProgramErrorKind::StackError(1))
        } else {
            Ok(unsafe { &*self.ptr.as_ptr().add(self.len - 1) })
        }
    }
    pub fn last_mut(&mut self) -> Result<&mut T, ProgramErrorKind> {
        if self.len == 0 {
            Err(ProgramErrorKind::StackError(1))
        } else {
            Ok(unsafe { &mut *self.ptr.as_ptr().add(self.len - 1) })
        }
    }
    pub fn last_mut_option(&mut self) -> Option<&mut T> {
        if self.len == 0 {
            None
        } else {
            unsafe { Some(&mut *self.ptr.as_ptr().add(self.len - 1)) }
        }
    }
    pub fn last_option(&mut self) -> Option<&T> {
        if self.len == 0 {
            None
        } else {
            unsafe { Some(&*self.ptr.as_ptr().add(self.len - 1)) }
        }
    }

    pub unsafe fn last_n(&self, n: usize) -> Result<&[T], ProgramErrorKind> {
        unsafe {
            if n > self.len {
                Err(ProgramErrorKind::StackError(n))
            } else {
                let nth = &*self.ptr.as_ptr().add(self.len - n);
                Ok(std::slice::from_raw_parts(nth, n))
            }
        }
    }

    pub unsafe fn at_most_n(&self, n: usize) -> Result<&[T], ProgramErrorKind> {
        unsafe {
            let num = n.min(self.len);
            let nth = &*self.ptr.as_ptr().add(self.len - num);
            Ok(std::slice::from_raw_parts(nth, num))
        }
    }

    pub fn iter<'a>(&self) -> StackIter<'a, T> {
        unsafe {
            let me = ManuallyDrop::new(self);
            let begin = me.ptr.as_ptr();
            let end = begin.add(me.len) as *mut T;
            StackIter {
                ptr: NonNull::new_unchecked(me.ptr.as_ptr() as *mut T),
                begin: NonNull::new_unchecked(me.ptr.as_ptr() as *mut T),
                cap: me.cap,
                end,
                _marker: PhantomData,
            }
        }
    }

    pub fn iter_mut<'a>(&mut self) -> StackIterMut<'a, T> {
        unsafe {
            let me = ManuallyDrop::new(self);
            let begin = me.ptr.as_ptr();
            let end = begin.add(me.len) as *mut T;
            StackIterMut {
                ptr: NonNull::new_unchecked(me.ptr.as_ptr() as *mut T),
                begin: NonNull::new_unchecked(me.ptr.as_ptr() as *mut T),
                cap: me.cap,
                end,
                _marker: PhantomData,
            }
        }
    }

    pub fn swap(&mut self) {
        unsafe { self.ptr.swap(self.ptr.add(1)) };
    }
}

impl<T> Drop for Stack<T> {
    fn drop(&mut self) {
        if self.cap != 0 {
            self.len = 0;
            let layout = Layout::array::<T>(self.cap).unwrap();
            unsafe {
                alloc::dealloc(self.ptr.as_ptr() as *mut u8, layout);
            }
        }
    }
}

pub struct StackIterMut<'a, T> {
    begin: NonNull<T>,
    ptr: NonNull<T>,
    end: *const T,
    cap: usize,
    _marker: PhantomData<&'a T>,
}

impl<'a, T> Iterator for StackIterMut<'a, T> {
    type Item = &'a mut T;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr.as_ptr() != self.end.cast_mut() {
            let old = self.ptr;
            self.ptr = unsafe { old.add(1) };
            Some(unsafe { self.ptr.as_mut() })
        } else {
            None
        }
    }
}

impl<'a, T> DoubleEndedIterator for StackIterMut<'a, T> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.ptr.as_ptr() != self.end.cast_mut() {
            unsafe {
                self.end = self.end.sub(1);
                Some(self.end.cast_mut().as_mut()?)
            }
        } else {
            None
        }
    }
}

pub struct StackIter<'a, T> {
    begin: NonNull<T>,
    ptr: NonNull<T>,
    end: *const T,
    cap: usize,
    _marker: PhantomData<&'a T>,
}

impl<'a, T> Iterator for StackIter<'a, T> {
    type Item = T;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr.as_ptr() != self.end.cast_mut() {
            let old = self.ptr;
            self.ptr = unsafe { old.add(1) };
            Some(unsafe { self.ptr.read() })
        } else {
            None
        }
    }
}

impl<'a, T> DoubleEndedIterator for StackIter<'a, T> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.ptr.as_ptr() != self.end.cast_mut() {
            unsafe {
                self.end = self.end.sub(1);
                Some(self.end.read())
            }
        } else {
            None
        }
    }
}
// impl<T> Deref for Stack<T> {
//     type Target = T;

//     fn deref(&self) -> &Self::Target {
//         unsafe { &self.ptr.as_ref() }
//     }
// }

// impl<T> DerefMut for Stack<T> {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         todo!()
//     }
// }
