pub mod chunk;
pub mod dropless;

pub type Dropless = dropless::Dropless;

#[inline(always)]
fn align_down(val: usize, align: usize) -> usize {
    debug_assert!(align.is_power_of_two());
    val & !(align - 1)
}

#[inline(always)]
fn align_up(val: usize, align: usize) -> usize {
    debug_assert!(align.is_power_of_two());
    (val + align - 1) & !(align - 1)
}
