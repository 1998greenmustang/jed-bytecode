use std::{
    alloc::{self, Layout},
    marker::PhantomData,
    mem::ManuallyDrop,
    ptr::{self, NonNull},
};

use crate::error::ProgramErrorKind;

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub struct List<T> {
    ptr: NonNull<T>,
    cap: usize,
    len: usize,
}

impl<T: std::fmt::Display> std::fmt::Display for List<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for i in 0..self.len {
            if i != self.len - 1 {
                unsafe { write!(f, "{}, ", self.ptr.add(i).as_ref())? };
            } else {
                unsafe { write!(f, "{}", self.ptr.add(i).as_ref())? };
            }
        }
        write!(f, "]")
    }
}

unsafe impl<T: Send> Send for List<T> {}

unsafe impl<T: Sync> Sync for List<T> {}

impl<T> Default for List<T> {
    fn default() -> Self {
        Self {
            ptr: NonNull::dangling(),
            cap: Default::default(),
            len: Default::default(),
        }
    }
}

impl<T> List<T> {
    pub fn new() -> List<T> {
        let mut this: List<T> = List {
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

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            Some(unsafe { self.ptr.add(self.len).read() })
        }
    }

    pub fn pop_if(&mut self, predicate: impl FnOnce(&mut T) -> bool) -> Option<T> {
        let last = self.last_mut()?;
        if predicate(last) {
            Some(unsafe { self.ptr.add(self.len).read() })
        } else {
            None
        }
    }

    pub fn pop_mut(&mut self) -> Option<&mut T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            Some(unsafe { self.ptr.add(self.len).as_mut() })
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn last_mut(&mut self) -> Option<&mut T> {
        if self.len == 0 {
            None
        } else {
            unsafe { Some(self.ptr.add(self.len - 1).as_mut()) }
        }
    }
    pub fn last(&mut self) -> Option<&T> {
        if self.len == 0 {
            None
        } else {
            unsafe { Some(self.ptr.add(self.len - 1).as_ref()) }
        }
    }

    pub unsafe fn last_n(&self, n: usize) -> Option<&[T]> {
        unsafe {
            if n > self.len {
                None
            } else {
                let nth = self.ptr.add(self.len - n).as_ref();
                Some(std::slice::from_raw_parts(nth, n))
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

    pub fn iter<'a>(&self) -> ListIter<'a, T> {
        unsafe {
            let me = ManuallyDrop::new(self);
            let end = me.ptr.as_ptr().add(me.len) as *const T;
            ListIter {
                ptr: me.ptr.as_ptr() as *const T,
                end,
                _marker: PhantomData,
            }
        }
    }

    pub fn iter_mut<'a>(&mut self) -> ListIterMut<'a, T> {
        unsafe {
            let me = ManuallyDrop::new(self);
            let end = me.ptr.as_ptr().add(me.len) as *const T;
            ListIterMut {
                ptr: me.ptr.as_ptr() as *const T,
                end,
                _marker: PhantomData,
            }
        }
    }

    pub fn first(&self) -> Option<&T> {
        if self.len == 0 {
            None
        } else {
            unsafe { Some(self.ptr.as_ref()) }
        }
    }

    pub unsafe fn get_unchecked(&self, idx: usize) -> &T {
        unsafe { self.ptr.add(idx).as_ref() }
    }

    pub unsafe fn get_unchecked_mut(&self, idx: usize) -> &mut T {
        unsafe { self.ptr.add(idx).as_mut() }
    }

    pub fn get(&self, idx: usize) -> Option<&T> {
        if self.len > idx {
            Some(unsafe { self.ptr.add(idx).as_ref() })
        } else {
            None
        }
    }

    pub fn get_mut(&self, idx: usize) -> Option<&mut T> {
        if self.len > idx {
            Some(unsafe { self.ptr.add(idx).as_mut() })
        } else {
            None
        }
    }

    pub fn insert(&self, idx: usize, item: &T) {
        if self.len > idx {
            unsafe { std::ptr::copy(item, self.ptr.add(idx).as_mut(), 1) }
        } else {
            panic!(
                "no such index {idx} in a list with length of {}",
                self.len()
            );
        }
    }

    pub fn remove(&mut self, idx: usize) -> T {
        if self.len > idx {
            let item = unsafe { self.ptr.add(idx).read() };
            self.len -= 1;
            if self.len - 1 != idx {
                // move all other items back
                for i in idx..self.len - 1 {
                    let elem = unsafe { self.ptr.add(i).read() };
                    unsafe { ptr::write(self.ptr.as_ptr().add(idx), elem) };
                }
            }
            item
        } else {
            panic!("no element at index {idx}");
        }
    }

    pub fn alloc(&mut self, n: usize) {
        while self.cap < n {
            self.grow()
        }
    }
}

impl<T> Drop for List<T> {
    #[track_caller]
    fn drop(&mut self) {
        // let caller_location = std::panic::Location::caller();
        // let caller_line_number = caller_location.line();
        // println!(
        //     "called from line: {}; {}",
        //     caller_line_number, caller_location
        // );
        println!("\n\tDROP !{:?} {} {}\n", self.ptr, self.cap, self.len);
        // println!("maybere");
        if self.cap != 0 {
            self.len = 0;
            let layout = Layout::array::<T>(self.cap).unwrap();
            unsafe {
                alloc::dealloc(self.ptr.as_ptr() as *mut u8, layout);
            }
            // println!("here");
        }
    }
}

pub struct ListIterMut<'a, T> {
    ptr: *const T,
    end: *const T,
    _marker: PhantomData<&'a T>,
}

impl<'a, T> Iterator for ListIterMut<'a, T> {
    type Item = &'a mut T;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr != self.end {
            let old = self.ptr;
            self.ptr = unsafe { old.add(1) };
            Some(unsafe { &mut *old.cast_mut() })
        } else {
            None
        }
    }
}

impl<'a, T> DoubleEndedIterator for ListIterMut<'a, T> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.ptr != self.end {
            unsafe {
                self.end = self.end.sub(1);
                Some(self.end.cast_mut().as_mut()?)
            }
        } else {
            None
        }
    }
}

pub struct ListIter<'a, T> {
    ptr: *const T,
    end: *const T,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: std::fmt::Debug> Iterator for ListIter<'a, T> {
    type Item = &'a T;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr != self.end {
            let old = self.ptr;
            self.ptr = unsafe { old.add(1) };
            unsafe { old.as_ref() }
        } else {
            None
        }
    }
}

impl<'a, T: std::fmt::Debug> DoubleEndedIterator for ListIter<'a, T> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.ptr != self.end {
            unsafe {
                self.end = self.end.sub(1);
                self.end.as_ref()
            }
        } else {
            None
        }
    }
}
// impl<T> Deref for List<T> {
//     type Target = T;

//     fn deref(&self) -> &Self::Target {
//         unsafe { &self.ptr.as_ref() }
//     }
// }

// impl<T> DerefMut for List<T> {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         todo!()
//     }
// }
