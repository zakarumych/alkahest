use crate::{
    deserialize::{Deserialize, DeserializeError, Deserializer},
    formula::{formula_fast_sizes, repeat_size, BareFormula, Formula},
    serialize::{field_size_hint, Serialize, Serializer},
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
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        <Self as Serialize<[F]>>::serialize(self, ser)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<(usize, usize)> {
        <Self as Serialize<[F]>>::size_hint(self)
    }
}

impl<'ser, F, T, const N: usize> Serialize<[F; N]> for &'ser [T; N]
where
    F: Formula,
    &'ser T: Serialize<F>,
{
    #[inline(always)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        <Self as Serialize<[F]>>::serialize(self, ser)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<(usize, usize)> {
        <Self as Serialize<[F]>>::size_hint(self)
    }
}

impl<F, T, const N: usize> Serialize<[F]> for [T; N]
where
    F: Formula,
    T: Serialize<F>,
{
    #[inline(always)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut ser = ser.into();
        self.into_iter()
            .try_for_each(|elem: T| ser.write_value::<F, T>(elem))?;
        ser.finish()
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<(usize, usize)> {
        if let Some(size) = formula_fast_sizes::<[F]>() {
            return Some(size);
        }
        if N <= 4 {
            let mut total_heap = 0;
            let mut total_stack = 0;
            for elem in self.iter() {
                let (heap, stack) = field_size_hint::<F>(elem, false)?;
                total_heap += heap;
                total_stack += stack;
            }
            return Some((total_heap, total_stack));
        }
        None
    }
}

impl<'ser, F, T, const N: usize> Serialize<[F]> for &'ser [T; N]
where
    F: Formula,
    &'ser T: Serialize<F>,
{
    #[inline(always)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut ser = ser.into();
        self.iter()
            .try_for_each(|elem: &T| ser.write_value::<F, &T>(elem))?;
        ser.finish()
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<(usize, usize)> {
        if let Some(size) = formula_fast_sizes::<[F]>() {
            return Some(size);
        }
        if N <= 4 {
            let mut total_heap = 0;
            let mut total_stack = 0;
            for elem in self.iter() {
                let (heap, stack) = field_size_hint::<F>(&elem, false)?;
                total_heap += heap;
                total_stack += stack;
            }
            return Some((total_heap, total_stack));
        }
        None
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
