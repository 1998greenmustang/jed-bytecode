use std::{alloc::Layout, ptr::NonNull};

pub mod chunk;
pub mod dropless;
pub mod manual;

pub type Dropless = dropless::Dropless;
pub type Manual<T> = manual::Manual<T>;

// The arenas start with PAGE-sized chunks, and then each new chunk is twice as
// big as its predecessor, up until we reach HUGE_PAGE-sized chunks, whereupon
// we stop growing. This scales well, from arenas that are barely used up to
// arenas that are used for 100s of MiBs. Note also that the chosen sizes match
// the usual sizes of pages and huge pages on Linux.
const PAGE: usize = 4096;
const HUGE_PAGE: usize = 2 * 1024 * 1024;

// Utility functions to help align
// I haven't been bothered to understand them
// Essentially they do magic

#[inline(always)]
fn align_down(val: usize, align: usize) -> usize {
    debug_assert!(align.is_power_of_two());
    val & !(align - 1)
}

// align_up(start + (Self::SIZE_OF_T * layout.size()), Self::ALIGN_OF_T) - 1;
#[inline(always)]
fn align_up(val: usize, align: usize) -> usize {
    debug_assert!(align.is_power_of_two());
    (val + align - 1) & !(align - 1)
}
