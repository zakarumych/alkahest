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
    const EXACT_SIZE: bool = match F::MAX_STACK_SIZE {
        Some(0) => true,
        _ => false,
    };
    const HEAPLESS: bool = match F::MAX_STACK_SIZE {
        Some(0) => true,
        _ => false,
    };
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

    fn size_hint(&self) -> Option<(usize, usize)> {
        Some((0, default_iter_fast_sizes::<F, _>(&self.iter())?))
    }
}

#[inline(always)]
pub fn default_iter_fast_sizes<F, I>(iter: &I) -> Option<usize>
where
    F: Formula,
    I: Iterator,
{
    match (F::EXACT_SIZE, F::HEAPLESS, F::MAX_STACK_SIZE) {
        (_, true, Some(0)) => Some(size_of::<FixedUsize>()),
        (_, true, Some(max_stack_size)) => {
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
