use crate::arena::{align_down, align_up};

use super::chunk::Chunk;
use super::{HUGE_PAGE, PAGE};
use std::slice;
use std::{
    alloc::Layout,
    cell::{Cell, RefCell},
    cmp, mem, ptr,
};

#[derive(Debug)]
struct Free<T> {
    start: Cell<*mut T>,
    size: usize,
}

impl<T> Free<T> {
    unsafe fn end(&self) -> *mut T {
        self.start.as_ptr().add(self.size) as *mut T
    }
}

pub struct Manual<T = u8> {
    start: Cell<*mut T>,
    endaa: Cell<*mut T>,
    chunks: RefCell<Vec<Chunk<T>>>,
    free: RefCell<Vec<Free<T>>>,
}

impl<T> Default for Manual<T> {
    fn default() -> Self {
        Manual {
            start: Cell::new(ptr::null_mut()),
            endaa: Cell::new(ptr::null_mut()),
            chunks: Default::default(),
            free: Default::default(),
        }
    }
}

impl<T> Manual<T> {
    pub const ALIGN_OF_T: usize = align_of::<T>();
    pub const SIZE_OF_T: usize = size_of::<T>();
    const PAGE_MAX_ENTRIES: usize = (PAGE / (Self::SIZE_OF_T + Self::ALIGN_OF_T - 1));
    const HUGE_PAGE_MAX_ENTRIES: usize = (HUGE_PAGE / (Self::SIZE_OF_T + Self::ALIGN_OF_T - 1));

    fn update_chunks(&self, ptr: *mut T, entry_count: isize) {
        let mut chunks = self.chunks.borrow_mut();
        let altered_chunk = chunks
            .iter_mut()
            .rfind(|x| x.storage.as_ptr().addr() <= ptr.addr());
        if let Some(chk) = altered_chunk {
            if entry_count < 0 {
                chk.entries -= entry_count as usize
            } else {
                chk.entries += entry_count as usize;
            }
        }
    }

    /// find empty joints within the pages or append the bottom
    pub fn allocate(&self, layout: Layout) -> *mut T {
        assert_ne!(layout.size(), 0);
        let entries = layout.size() / Self::SIZE_OF_T;
        // Search through free memory
        let mut free = self.free.borrow_mut();

        // Prefered ==
        if let Some(entry_idx) = free
            .iter_mut()
            .position(|x| x.size == layout.size())
            .as_mut()
        {
            let entry = unsafe { free.get_unchecked(*entry_idx) };
            let start = entry.start.clone().into_inner();
            free.remove(*entry_idx);
            self.update_chunks(start, layout.size() as isize);
            return start;
        }

        // Then try out >
        if let Some(entry_idx) = free
            .iter_mut()
            .position(|x| x.size > layout.size())
            .as_mut()
        {
            let entry = unsafe { free.get_unchecked_mut(*entry_idx) };
            let start = entry.start.clone().into_inner();
            entry.size -= layout.size();
            self.update_chunks(start, layout.size() as isize);
            return start;
        }

        // allocate from `self.start` to `self.start + layout.size()`
        // return self.start; change self.start to `self.start + layout.size()`
        loop {
            let old_start = self.start.get();
            let start = old_start.addr();
            let end = self.endaa.get().addr();

            let bytes = align_up(layout.size(), Self::ALIGN_OF_T);
            match start.checked_add(bytes) {
                Some(add) => {
                    let new_start = align_up(add, layout.align());
                    if new_start <= end {
                        let new_start = old_start.with_addr(new_start);
                        self.start.set(new_start);
                        self.update_chunks(old_start, layout.size() as isize);
                        return old_start;
                    }
                }
                _ => (),
            }

            self.grow(layout);
        }
    }

    pub fn deallocate(&self, ptr: *mut T, layout: Layout) {
        assert_eq!(ptr.align_offset(layout.align()), 0);
        let start = ptr.addr();
        let end = ptr.addr() + layout.size();
        // dbg!(
        //     endaa,
        //     start,
        //     layout.size(),
        //     &self.start.get().addr(),
        //     &self.endaa.get().addr(),
        //     endaa >= self.start.get().addr(),
        //     start <= self.endaa.get().addr()
        // );

        if start < self.start.get().addr() {
            // find chunks we are altering and remove the entries
            let mut chunks = self.chunks.borrow_mut();
            let altered_chunks: Vec<&mut Chunk<T>> = chunks
                .iter_mut()
                .filter(|x| {
                    let x_stt = { x.storage.as_ptr() as *mut T }.addr();
                    let x_end = x_stt + x.storage.len();
                    x_stt <= end || x_end >= start
                })
                .collect();
            assert_ne!(
                altered_chunks.len(),
                0,
                "Cannot find the chunk being altered"
            );
            for chk in altered_chunks {
                let chk_start = chk.start().addr();
                // this chunk is being changed
                // get the range being changed
                // `start`..=`chk.end()` or `start`..=end
                let chk_end = chk.end().addr().min(end);
                // dbg!(chk_end, chk_start);
                if let Some(size) = chk_end.checked_sub(start) {
                    let entries_count = size / Self::SIZE_OF_T + 1;

                    // dbg!(chk.entries, entries_count, size);
                    chk.entries = chk
                        .entries
                        .checked_sub(entries_count)
                        .unwrap_or_else(|| panic!("impossible deallocation"));
                    // dbg!(chk.entries);
                } else {
                    unreachable!("i think i asserted enough")
                }
            }

            self.add_free(ptr.into(), layout.size());
        }

        // i want to shrink here but im not sure how
        // if self.end.get().addr() == end {
        //     self.shrink();
        // }
    }

    fn add_free(&self, start: Cell<*mut T>, size: usize) {
        let mut free = self.free.borrow_mut();
        if let Some(entry) = free.iter_mut().find(|x| x.start == start).as_mut() {
            entry.size += size;
        } else {
            free.push(Free { start, size })
        }
    }

    pub fn start(&self) -> *mut T {
        self.start.get()
    }

    pub fn grow(&self, layout: Layout) {
        // this is to ensure our memory is aligned properly
        let padding = layout.size() + cmp::max(Self::ALIGN_OF_T, layout.align());
        let mut chunks = self.chunks.borrow_mut();
        let mut new_cap;

        if let Some(last_chunk) = chunks.last_mut() {
            // This adds either a PAGE or a HUGE_PAGE to our capacity
            // Once we add a HUGE_PAGE, we will continue adding those instead
            new_cap = last_chunk.storage.len().min(HUGE_PAGE / 2);
            new_cap *= 2;
        } else {
            new_cap = PAGE;
        }

        // Ensurement of alignment again
        new_cap = cmp::max(padding, new_cap);

        let mut chunk = Chunk::new(align_up(new_cap, PAGE));
        self.start.set(chunk.start());

        // Align the end of the chunk properly
        let end = align_down(chunk.end().addr(), align_of::<T>());

        // Make sure we're not dumb, `end` should be greater than `start`
        debug_assert!(chunk.start().addr() <= end);

        self.endaa.set(chunk.end().with_addr(end));

        chunks.push(chunk);
    }

    /// Remove necessary chunks based off of (equivalent) entry count or layout size
    /// this is definitely not implemented right
    pub fn shrink(&self, layout: Layout) {
        assert!(layout.size() > 0);

        // ensure the memory to remove is aligned to T
        let true_size = cmp::max(layout.size(), Self::SIZE_OF_T)
            + cmp::max(layout.align(), Self::ALIGN_OF_T)
            - 1;

        let mut chunks = self.chunks.borrow_mut();
        let mut size_to_remove = true_size.clone();
        let mut entries_equivalent = size_to_remove / Self::SIZE_OF_T;

        // see if we can't remove a chunk(s)
        // remove chunks until we have less than a chunk size
        while let Some(last) = chunks
            .pop_if(|x| (x.storage.len() <= size_to_remove) || (x.entries <= entries_equivalent))
            .as_mut()
        {
            size_to_remove = size_to_remove.checked_sub(last.storage.len()).unwrap_or(0);
            entries_equivalent -= last.entries;
        }

        // We must've removed everything by getting down here
        assert_eq!(size_to_remove, 0);

        if let Some(last_chunk) = chunks.last_mut() {
            let end = align_down(last_chunk.end().addr(), Self::ALIGN_OF_T);
            self.endaa.set(last_chunk.end().with_addr(end));
        } else {
            self.start.set(ptr::null_mut());
            self.endaa.set(ptr::null_mut());
        }
    }

    pub fn alloc_slice(&self, slice: &[T]) -> &mut [T] {
        assert!(!mem::needs_drop::<T>());
        assert!(Self::SIZE_OF_T != 0);
        assert!(!slice.is_empty());

        let mem = self.allocate(Layout::for_value::<[T]>(slice)) as *mut T;

        assert_eq!(mem.align_offset(Self::ALIGN_OF_T), 0);
        unsafe {
            mem.copy_from(slice.as_ptr(), slice.len());
            return slice::from_raw_parts_mut(mem, slice.len());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const U8_VALUE: u8 = 127;
    const U8_ALIGNMENT: usize = align_of::<u8>();

    #[test]
    fn create_manual() {
        let manual: Manual<u8> = Default::default();
        assert!(manual.start.get().is_null());
        assert!(manual.endaa.get().is_null());
        assert!(manual.chunks.borrow().len() == 0);
    }

    #[test]
    fn grow_then_shrink_page_wise() {
        let manual: Manual<u8> = Default::default();

        manual.grow(Layout::for_value(&U8_VALUE));
        assert_eq!(manual.chunks.borrow().first().unwrap().storage.len(), PAGE);
        assert!(!manual.start.get().is_null());
        assert!(!manual.endaa.get().is_null());

        manual.shrink(Layout::from_size_align(PAGE, U8_ALIGNMENT).ok().unwrap());
        assert!(manual.start.get().is_null());
        assert!(manual.endaa.get().is_null());
    }

    #[test]
    fn alloc_then_dealloc() {
        let manual: Manual<u8> = Default::default();

        let ptr1 = manual.allocate(Layout::for_value(&U8_VALUE));
        assert_eq!(manual.chunks.borrow().first().unwrap().entries, 1);

        let _ptr2 = manual.allocate(Layout::for_value(&U8_VALUE));
        assert_eq!(manual.free.borrow().len(), 0);
        assert_eq!(manual.chunks.borrow().first().unwrap().entries, 2);

        manual.deallocate(ptr1, Layout::for_value(&U8_VALUE));
        assert_eq!(manual.free.borrow().len(), 1);
        assert_eq!(manual.chunks.borrow().first().unwrap().entries, 1);
        // assert!(manual.start.get().is_null());
        // assert!(manual.end.get().is_null());
    }

    #[test]
    fn alloc_slice() {
        let manual: Manual<u8> = Default::default();

        let slice = [U8_VALUE; PAGE];
        let saved_bytes = manual.alloc_slice(&slice);
        let saved_bytes: &'static [u8] = unsafe { &*(saved_bytes as *const [u8]) };

        assert_eq!(manual.chunks.borrow().len(), 1);
        saved_bytes.iter().for_each(|x| assert_eq!(x, &U8_VALUE));
        // dbg!(
        //     saved_bytes.as_ptr(),
        //     saved_bytes.first().unwrap() as *const u8, // first addr
        //     saved_bytes.last().unwrap() as *const u8 as usize, // last addr
        //     manual
        //         .chunks
        //         .borrow()
        //         .iter()
        //         .map(|x| x.storage.as_ptr())
        //         .collect::<Vec<*mut [mem::MaybeUninit<u8>]>>()
        // );

        // assert_eq!();
    }
    #[test]
    fn alloc_slice_then_dealloc_some_then_alloc_more() {
        let manual: Manual<u8> = Default::default();

        let slice = [U8_VALUE; PAGE];
        let saved_bytes = manual.alloc_slice(&slice);
        let saved_bytes: &'static [u8] = unsafe { &*(saved_bytes as *const [u8]) };
        let saved_addr = saved_bytes.as_ptr();

        // test the copy
        saved_bytes.iter().for_each(|x| assert_eq!(x, &U8_VALUE));

        // "remove" half of the slice
        manual.deallocate(
            saved_addr.cast_mut(),
            Layout::for_value(&slice[0..PAGE / 2]),
        );

        assert_eq!(
            manual.chunks.borrow().first().unwrap().entries,
            PAGE / 2 - 1,
            "chunk.entries updated"
        );

        // this should still pass, even though it was deallocated
        saved_bytes.iter().for_each(|x| assert_eq!(x, &U8_VALUE));

        let slice2 = [U8_VALUE - 64; PAGE / 3];
        let saved_slice2 = manual.alloc_slice(&slice2);
        let saved_slice2: &'static [u8] = unsafe { &*(saved_slice2 as *const [u8]) };

        saved_slice2
            .iter()
            .for_each(|x| assert_eq!(x, &(U8_VALUE - 64)));

        saved_bytes[PAGE / 2..]
            .iter()
            .for_each(|x| assert_eq!(x, &U8_VALUE));

        // Half of the bytes were removed
        // A third of the bytes have been changed
        saved_bytes[..PAGE / 3]
            .iter()
            .for_each(|x| assert_eq!(x, &(U8_VALUE - 64)));

        assert_eq!(manual.free.borrow().first().unwrap().size, 683);

        let slice3 = [U8_VALUE + 64; PAGE / 6 + 1];
        let saved_slice3 = manual.alloc_slice(&slice3);
        let saved_slice3: &'static [u8] = unsafe { &*(saved_slice3 as *const [u8]) };

        saved_slice3
            .iter()
            .for_each(|x| assert_eq!(x, &(U8_VALUE + 64)));
        dbg!(manual.free.borrow().first(), slice3.len());
        assert!(manual.free.borrow().first().is_none());
    }
}
