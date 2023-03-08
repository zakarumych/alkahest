use core::mem::size_of;

use crate::{
    formula::{BareFormula, Formula},
    serialize::{Serialize, Serializer},
    size::FixedUsize,
};

impl<F> Formula for [F]
where
    F: Formula,
{
    const MAX_STACK_SIZE: Option<usize> = match F::MAX_STACK_SIZE {
        Some(0) => Some(size_of::<FixedUsize>()),
        Some(_) => None,
        None => None,
    };
    const EXACT_SIZE: bool = false;
    const HEAPLESS: bool = F::HEAPLESS;
}

impl<F> BareFormula for [F] where F: Formula {}

impl<'ser, F, T> Serialize<[F]> for &'ser [T]
where
    F: Formula,
    &'ser T: Serialize<F>,
{
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        Self: Sized,
        S: Serializer,
    {
        let mut ser = ser.into();
        ser.write_slice::<F, &'ser T>(self.iter())?;
        ser.finish()
    }

    fn size_hint(&self) -> Option<usize> {
        default_iter_fast_sizes_owned::<F, &'ser T, _>(self.iter())
    }
}

#[inline(always)]
pub fn default_iter_fast_sizes<F, I>(iter: &I) -> Option<usize>
where
    F: Formula,
    I: Iterator,
    I::Item: Serialize<F>,
{
    match (F::EXACT_SIZE, F::HEAPLESS, F::MAX_STACK_SIZE) {
        (true, true, Some(max_stack_size)) => {
            let (lower, upper) = iter.size_hint();
            match upper {
                Some(upper) if upper == lower => {
                    // Expect this to be the truth.
                    // If not, serialization will fail or produce incorrect results.
                    Some(lower * max_stack_size)
                }
                _ => None,
            }
        }
        _ => None,
    }
}

#[inline(always)]
pub fn default_iter_fast_sizes_owned<F, T, I>(iter: I) -> Option<usize>
where
    F: Formula,
    I: Iterator<Item = T>,
    T: Serialize<F>,
{
    default_iter_fast_sizes_impl::<F, I>(iter, |e| <T as Serialize<F>>::size_hint(&e))
}

#[inline(always)]
pub fn default_iter_fast_sizes_by_ref<'a, F, T, I>(iter: I) -> Option<usize>
where
    F: Formula,
    I: Iterator<Item = &'a T>,
    T: Serialize<F> + 'a,
{
    default_iter_fast_sizes_impl::<F, I>(iter, <T as Serialize<F>>::size_hint)
}

#[inline(always)]
pub fn default_iter_fast_sizes_impl<'a, F, I>(
    iter: I,
    size_hint: impl Fn(I::Item) -> Option<usize>,
) -> Option<usize>
where
    F: Formula,
    I: Iterator,
{
    match (F::EXACT_SIZE, F::HEAPLESS, F::MAX_STACK_SIZE) {
        (true, true, Some(max_stack_size)) => {
            let (lower, upper) = iter.size_hint();
            match upper {
                Some(upper) if upper == lower => {
                    // Expect this to be the truth.
                    // If not, serialization will fail or produce incorrect results.
                    Some(lower * max_stack_size)
                }
                _ => None,
            }
        }
        _ => {
            let (_, upper) = iter.size_hint();
            match upper {
                Some(upper) if upper <= 4 => {
                    let mut size = 0;
                    for elem in iter {
                        if F::MAX_STACK_SIZE.is_none() {
                            size += size_of::<FixedUsize>();
                        }
                        size += size_hint(elem)?;
                    }
                    Some(size)
                }
                _ => None,
            }
        }
    }
}
