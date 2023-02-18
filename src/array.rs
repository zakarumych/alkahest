use core::mem::size_of;

use crate::{
    deserialize::{Deserialize, Deserializer, Error},
    formula::{formula_fast_sizes, repeat_size, BareFormula, Formula},
    serialize::{Serialize, Serializer},
    size::FixedUsize,
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
    #[inline(never)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut ser = ser.into();
        self.into_iter()
            .try_for_each(|elem: T| ser.write_value::<F, T>(elem))?;
        ser.finish()
    }

    #[inline(never)]
    fn size_hint(&self) -> Option<usize> {
        if let Some(size) = formula_fast_sizes::<[F; N]>() {
            return Some(size);
        }
        if N <= 4 {
            let mut size = 0;
            for elem in self.iter() {
                size += elem.size_hint()?;
            }
            Some(size)
        } else {
            None
        }
    }
}

impl<'ser, F, T, const N: usize> Serialize<[F; N]> for &'ser [T; N]
where
    F: Formula,
    &'ser T: Serialize<F>,
{
    #[inline(never)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut ser = ser.into();
        self.iter()
            .try_for_each(|elem: &T| ser.write_value::<F, &T>(elem))?;
        ser.finish()
    }

    #[inline(never)]
    fn size_hint(&self) -> Option<usize> {
        if let Some(size) = formula_fast_sizes::<[F; N]>() {
            return Some(size);
        }
        if N <= 4 {
            let mut size = 0;
            for elem in self.iter() {
                size += (&elem).size_hint()?;
            }
            Some(size)
        } else {
            None
        }
    }
}

impl<F, T, const N: usize> Serialize<[F]> for [T; N]
where
    F: Formula,
    T: Serialize<F>,
{
    #[inline(never)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut ser = ser.into();
        self.into_iter()
            .try_for_each(|elem: T| ser.write_value::<F, T>(elem))?;
        ser.finish()
    }

    #[inline(never)]
    fn size_hint(&self) -> Option<usize> {
        if let Some(size) = formula_fast_sizes::<[F]>() {
            return Some(size);
        }
        if N <= 4 {
            let mut size = 0;
            for elem in self.iter() {
                if F::MAX_STACK_SIZE.is_none() {
                    size += size_of::<FixedUsize>();
                }
                size += elem.size_hint()?;
            }
            Some(size)
        } else {
            None
        }
    }
}

impl<'ser, F, T, const N: usize> Serialize<[F]> for &'ser [T; N]
where
    F: Formula,
    &'ser T: Serialize<F>,
{
    #[inline(never)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut ser = ser.into();
        self.iter()
            .try_for_each(|elem: &T| ser.write_value::<F, &T>(elem))?;
        ser.finish()
    }

    #[inline(never)]
    fn size_hint(&self) -> Option<usize> {
        if let Some(size) = formula_fast_sizes::<[F]>() {
            return Some(size);
        }
        if N <= 4 {
            let mut size = 0;
            for elem in self.iter() {
                if F::MAX_STACK_SIZE.is_none() {
                    size += size_of::<FixedUsize>();
                }
                size += (&elem).size_hint()?;
            }
            Some(size)
        } else {
            None
        }
    }
}

impl<'de, F, T, const N: usize> Deserialize<'de, [F; N]> for [T; N]
where
    F: Formula,
    T: Deserialize<'de, F>,
{
    #[inline(never)]
    fn deserialize(mut de: Deserializer<'de>) -> Result<Self, Error> {
        let mut opts = [(); N].map(|_| None);
        opts.iter_mut().try_for_each(|slot| {
            *slot = Some(de.read_value::<F, T>(false)?);
            Ok(())
        })?;
        let value = opts.map(|slot| slot.unwrap());
        Ok(value)
    }

    #[inline(never)]
    fn deserialize_in_place(&mut self, mut de: Deserializer<'de>) -> Result<(), Error> {
        self.iter_mut()
            .try_for_each(|elem| de.read_in_place::<F, T>(elem, false))?;
        Ok(())
    }
}
