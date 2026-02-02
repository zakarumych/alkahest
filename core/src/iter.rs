use core::borrow::Borrow;

use crate::{
    formula::Formula,
    list::List,
    serialize::{Serialize, Serializer, Sizes},
};

/// Returns the size of the serialized data if it can be determined fast.
#[inline(always)]
pub fn iter_fast_sizes<'a, F, T, I>(iter: I, size_bytes: u8) -> Option<Sizes>
where
    F: Formula + ?Sized,
    T: Serialize<F> + 'a,
    I: Iterator,
    I::Item: Borrow<T>,
{
    let sizes = Sizes::with_stack(usize::from(size_bytes));

    match (F::HEAPLESS, F::max_stack_size(size_bytes)) {
        (true, Some(0)) => Some(sizes),
        (true, Some(max_stack)) => {
            let (lower, upper) = iter.size_hint();
            match upper {
                Some(upper) if upper == lower => {
                    // Expect this to be the truth.
                    // If not, serialization will fail or produce incorrect results.
                    Some(sizes + Sizes::with_stack(lower * max_stack))
                }
                _ => None,
            }
        }
        _ => None,
    }
}
