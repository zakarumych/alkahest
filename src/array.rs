use crate::{
    buffer::Buffer,
    deserialize::{Deserialize, DeserializeError, Deserializer},
    formula::{repeat_size, BareFormula, Formula},
    iter::{owned_iter_fast_sizes, ref_iter_fast_sizes},
    serialize::{write_slice, Serialize, Sizes},
};

impl<F, const N: usize> Formula for [F; N]
where
    F: Formula,
{
    const MAX_STACK_SIZE: Option<usize> = repeat_size(F::MAX_STACK_SIZE, N);
    const EXACT_SIZE: bool = F::EXACT_SIZE;
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
        <Self as Serialize<[F]>>::serialize(self, sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        <Self as Serialize<[F]>>::size_hint(self)
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
        <Self as Serialize<[F]>>::serialize(self, sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        <Self as Serialize<[F]>>::size_hint(self)
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
        owned_iter_fast_sizes::<F, _, _>(self.iter())
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
        ref_iter_fast_sizes::<F, _, _>(self.iter())
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
