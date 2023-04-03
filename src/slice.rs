use crate::{
    buffer::Buffer,
    formula::{BareFormula, Formula},
    iter::owned_iter_fast_sizes,
    serialize::{write_slice, Serialize, Sizes},
    size::SIZE_STACK,
};

impl<F> Formula for [F]
where
    F: Formula,
{
    const MAX_STACK_SIZE: Option<usize> = match F::MAX_STACK_SIZE {
        Some(0) => Some(SIZE_STACK),
        _ => None,
    };
    const EXACT_SIZE: bool = true; // All elements are padded.
    const HEAPLESS: bool = F::HEAPLESS;
}

impl<F> BareFormula for [F] where F: Formula {}

impl<'ser, F, T> Serialize<[F]> for &'ser [T]
where
    F: Formula,
    &'ser T: Serialize<F>,
{
    #[inline(always)]
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        Self: Sized,
        B: Buffer,
    {
        write_slice(self.iter(), sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        owned_iter_fast_sizes::<F, _, _>(self.iter())
    }
}
