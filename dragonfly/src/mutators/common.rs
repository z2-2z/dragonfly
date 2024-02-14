use libafl_bolts::prelude::Rand;
use std::ops::Range;

#[inline]
pub(crate) fn random_range<R: Rand>(rand: &mut R, limit: usize, max_size: usize) -> Range<usize> {
    debug_assert!(limit > 0);
    debug_assert!(max_size > 0);
    let start = rand.below(limit as u64) as usize;
    let rem_len = std::cmp::min(limit - start, max_size);
    let len = 1 + rand.below(rem_len as u64) as usize;
    debug_assert!(len <= max_size);
    start..start + len
}

#[inline]
pub(crate) fn copy_vec<T: Clone + Copy + Default>(to: &mut Vec<T>, from: &[T]) {
    to.resize(from.len(), T::default());
    to[..].copy_from_slice(from);
}
