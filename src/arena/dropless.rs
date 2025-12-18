// For use with ast::symbol::SymbolTable
// It's *so* similar, that I must use an arena.
// https://doc.rust-lang.org/beta/nightly-rustc/src/rustc_arena/lib.rs.html#350-366
//
// Rewriting here, for practice. For all intents and purposes, might as well be copied
//

use crate::arena::{align_down, align_up};

use super::chunk::Chunk;
use std::slice;
use std::{
    alloc::Layout,
    cell::{Cell, RefCell},
    cmp, mem, ptr,
};

pub struct Dropless {
    start: Cell<*mut u8>,
    end: Cell<*mut u8>,
    chunks: RefCell<Vec<Chunk>>,
}

impl Default for Dropless {
    fn default() -> Self {
        Dropless {
            start: Cell::new(ptr::null_mut()),
            end: Cell::new(ptr::null_mut()),
            chunks: Default::default(),
        }
    }
}

// The arenas start with PAGE-sized chunks, and then each new chunk is twice as
// big as its predecessor, up until we reach HUGE_PAGE-sized chunks, whereupon
// we stop growing. This scales well, from arenas that are barely used up to
// arenas that are used for 100s of MiBs. Note also that the chosen sizes match
// the usual sizes of pages and huge pages on Linux.
const PAGE: usize = 4096;
const HUGE_PAGE: usize = 2 * 1024 * 1024;
const DROPLESS_ALIGNMENT: usize = align_of::<usize>();

impl Dropless {
    #[inline(never)]
    #[cold]
    fn grow(&self, layout: Layout) {
        // Add some padding so we can align `self.end` while
        // still fitting in a `layout` allocation.
        let additional = layout.size() + cmp::max(DROPLESS_ALIGNMENT, layout.align()) - 1;

        unsafe {
            let mut chunks = self.chunks.borrow_mut();
            let mut new_cap;
            if let Some(last_chunk) = chunks.last_mut() {
                // There is no need to update `last_chunk.entries` because that
                // field isn't used by `DroplessArena`.

                // If the previous chunk's len is less than HUGE_PAGE
                // bytes, then this chunk will be least double the previous
                // chunk's size.
                new_cap = last_chunk.storage.len().min(HUGE_PAGE / 2);
                new_cap *= 2;
            } else {
                new_cap = PAGE;
            }
            // Also ensure that this chunk can fit `additional`.
            new_cap = cmp::max(additional, new_cap);

            let mut chunk = Chunk::new(align_up(new_cap, PAGE));
            self.start.set(chunk.start());

            // Align the end to DROPLESS_ALIGNMENT.
            let end = align_down(chunk.end().addr(), DROPLESS_ALIGNMENT);

            // Make sure we don't go past `start`. This should not happen since the allocation
            // should be at least DROPLESS_ALIGNMENT - 1 bytes.
            debug_assert!(chunk.start().addr() <= end);

            self.end.set(chunk.end().with_addr(end));

            chunks.push(chunk);
        }
    }

    #[inline]
    pub fn alloc_raw(&self, layout: Layout) -> *mut u8 {
        assert!(layout.size() != 0);

        // This loop executes once or twice: if allocation fails the first
        // time, the `grow` ensures it will succeed the second time.
        loop {
            let start = self.start.get().addr();
            let old_end = self.end.get();
            let end = old_end.addr();

            // Align allocated bytes so that `self.end` stays aligned to
            // DROPLESS_ALIGNMENT.
            let bytes = align_up(layout.size(), DROPLESS_ALIGNMENT);

            // Tell LLVM that `end` is aligned to DROPLESS_ALIGNMENT.
            // unsafe { intrinsics::assume(end == align_down(end, DROPLESS_ALIGNMENT)) };

            if let Some(sub) = end.checked_sub(bytes) {
                let new_end = align_down(sub, layout.align());
                if start <= new_end {
                    let new_end = old_end.with_addr(new_end);
                    // `new_end` is aligned to DROPLESS_ALIGNMENT as `align_down`
                    // preserves alignment as both `end` and `bytes` are already
                    // aligned to DROPLESS_ALIGNMENT.
                    self.end.set(new_end);
                    return new_end;
                }
            }

            // No free space left. Allocate a new chunk to satisfy the request.
            // On failure the grow will panic or abort.
            self.grow(layout);
        }
    }

    /// Allocates a slice of objects that are copied into the `arena::Dropless`, returning a mutable
    /// reference to it. Will panic if passed a zero-sized type.
    ///
    /// Panics:
    ///
    ///  - Zero-sized types
    ///  - Zero-length slices
    #[inline]
    pub fn alloc_slice<T>(&self, slice: &[T]) -> &mut [T]
    where
        T: Copy,
    {
        assert!(!mem::needs_drop::<T>());
        assert!(size_of::<T>() != 0);
        assert!(!slice.is_empty());

        let mem = self.alloc_raw(Layout::for_value::<[T]>(slice)) as *mut T;

        unsafe {
            mem.copy_from_nonoverlapping(slice.as_ptr(), slice.len());
            slice::from_raw_parts_mut(mem, slice.len())
        }
    }
}
