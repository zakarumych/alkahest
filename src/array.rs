use crate::{
    buffer::Buffer,
    deserialize::{Deserialize, DeserializeError, Deserializer},
    formula::{repeat_size, BareFormula, Formula},
    iter::{owned_iter_fast_sizes, ref_iter_fast_sizes},
    serialize::{write_array, write_slice, Serialize, Sizes},
};

impl<F, const N: usize> Formula for [F; N]
where
    F: Formula,
{
    const MAX_STACK_SIZE: Option<usize> = repeat_size(F::MAX_STACK_SIZE, N);
    const EXACT_SIZE: bool = true; // All elements are padded.
    const HEAPLESS: bool = F::HEAPLESS;
}

impl<F, const N: usize> BareFormula for [F; N] where F: Formula {}

impl<F, T, const N: usize> Serialize<[F; N]> for [T; N]
where
    F: Formula,
    T: Serialize<F>,
{
    #[inline(always)]
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        write_array(self.into_iter(), sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        ref_array_fast_sizes::<F, _, _>(self.iter())
    }
}

impl<'ser, F, T, const N: usize> Serialize<[F; N]> for &'ser [T; N]
where
    F: Formula,
    &'ser T: Serialize<F>,
{
    #[inline(always)]
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        write_array(self.iter(), sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        owned_array_fast_sizes::<F, _, _>(self.iter())
    }
}

impl<F, T, const N: usize> Serialize<[F]> for [T; N]
where
    F: Formula,
    T: Serialize<F>,
{
    #[inline(always)]
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        write_slice(self.into_iter(), sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        ref_iter_fast_sizes::<F, _, _>(self.iter())
    }
}

impl<'ser, F, T, const N: usize> Serialize<[F]> for &'ser [T; N]
where
    F: Formula,
    &'ser T: Serialize<F>,
{
    #[inline(always)]
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        write_slice(self.iter(), sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        owned_iter_fast_sizes::<F, _, _>(self.iter())
    }
}

impl<'de, F, T, const N: usize> Deserialize<'de, [F; N]> for [T; N]
where
    F: Formula,
    T: Deserialize<'de, F>,
{
    #[inline(always)]
    fn deserialize(mut de: Deserializer<'de>) -> Result<Self, DeserializeError> {
        let mut opts = [(); N].map(|_| None);
        opts.iter_mut().try_for_each(|slot| {
            *slot = Some(de.read_value::<F, T>(false)?);
            Ok(())
        })?;
        let value = opts.map(|slot| slot.unwrap());
        Ok(value)
    }

    #[inline(always)]
    fn deserialize_in_place(&mut self, mut de: Deserializer<'de>) -> Result<(), DeserializeError> {
        self.iter_mut()
            .try_for_each(|elem| de.read_in_place::<F, T>(elem, false))?;
        Ok(())
    }
}

/// Returns the size of the serialized data if it can be determined fast.
#[inline(always)]
pub fn owned_array_fast_sizes<F, I, T>(iter: I) -> Option<Sizes>
where
    F: Formula + ?Sized,
    I: Iterator<Item = T>,
    T: Serialize<F>,
{
    match (F::HEAPLESS, F::MAX_STACK_SIZE) {
        (true, Some(0)) => Some(Sizes::ZERO),
        _ => owned_iter_fast_sizes::<F, I, T>(iter),
    }
}

/// Returns the size of the serialized data if it can be determined fast.
#[inline(always)]
pub fn ref_array_fast_sizes<'a, F, I, T: 'a>(iter: I) -> Option<Sizes>
where
    F: Formula + ?Sized,
    I: Iterator<Item = &'a T>,
    T: Serialize<F>,
{
    match (F::HEAPLESS, F::MAX_STACK_SIZE) {
        (true, Some(0)) => Some(Sizes::ZERO),
        _ => ref_iter_fast_sizes::<F, I, T>(iter),
    }
}
