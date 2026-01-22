use std::{
    alloc::{self, Layout},
    error::Error,
    ptr::{self, NonNull},
};

use crate::error::ProgramErrorKind;

#[derive(Debug)]
pub struct Stack<T> {
    ptr: NonNull<T>,
    cap: usize,
    len: usize,
}
unsafe impl<T: Send> Send for Stack<T> {}

unsafe impl<T: Sync> Sync for Stack<T> {}

impl<T> Stack<T> {
    pub fn new() -> Stack<T> {
        let mut this = Stack {
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
    pub unsafe fn pop_n(&mut self, n: usize) -> Result<&[T], ProgramErrorKind> {
        if n > self.len {
            Err(ProgramErrorKind::StackError(n))
        } else {
            let nth = &*self.ptr.as_ptr().add(self.len - n);
            self.len -= n;
            Ok(std::slice::from_raw_parts(nth, n))
        }
    }

    pub fn len(&mut self) -> usize {
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
        if n > self.len {
            Err(ProgramErrorKind::StackError(n))
        } else {
            let nth = &*self.ptr.as_ptr().add(self.len - n);
            Ok(std::slice::from_raw_parts(nth, n))
        }
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
