use libafl_bolts::prelude::Rand;
use std::ops::Range;

#[inline]
pub(crate) fn random_range<R: Rand>(rand: &mut R, len: usize) -> Range<usize> {
    debug_assert!(len > 0);
    let start = rand.below(len as u64) as usize;
    let rem_len = len - start;
    let len = 1 + rand.below(rem_len as u64) as usize;
    start..start + len
}