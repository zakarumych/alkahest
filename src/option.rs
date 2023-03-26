use crate::{
    deserialize::{Deserialize, DeserializeError, Deserializer},
    formula::{sum_size, BareFormula, Formula},
    serialize::{field_size_hint, Serialize, Serializer},
};

impl<F> Formula for Option<F>
where
    F: Formula,
{
    const MAX_STACK_SIZE: Option<usize> = sum_size(Some(1), F::MAX_STACK_SIZE);
    const EXACT_SIZE: bool = false;
    const HEAPLESS: bool = true;
}

impl<F> BareFormula for Option<F> where F: Formula {}

impl<F, T> Serialize<Option<F>> for Option<T>
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
        match self {
            None => {
                ser.write_bytes(&[0u8])?;
                ser.finish()
            }
            Some(value) => {
                ser.write_bytes(&[1u8])?;
                ser.write_last_value::<F, T>(value)
            }
        }
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<(usize, usize)> {
        match self {
            None => {
                let stack = <Option<F>>::MAX_STACK_SIZE?;
                Some((0, stack))
            }
            Some(value) => {
                let (heap, stack) = field_size_hint::<F>(value, true)?;
                Some((heap, stack + 1))
            }
        }
    }
}

impl<'ser, F, T> Serialize<Option<F>> for &'ser Option<T>
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
        match self {
            None => {
                ser.write_bytes(&[0u8])?;
                ser.finish()
            }
            Some(value) => {
                ser.write_bytes(&[1u8])?;
                ser.write_last_value::<F, &'ser T>(value)
            }
        }
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<(usize, usize)> {
        match *self {
            None => {
                let stack = <Option<F>>::MAX_STACK_SIZE?;
                Some((0, stack))
            }
            Some(value) => {
                let (heap, stack) = field_size_hint::<F>(&value, true)?;
                Some((heap, stack + 1))
            }
        }
    }
}

impl<'de, F, T> Deserialize<'de, Option<F>> for Option<T>
where
    F: Formula,
    T: Deserialize<'de, F>,
{
    #[inline(always)]
    fn deserialize(mut de: Deserializer<'de>) -> Result<Self, DeserializeError> {
        let is_some: u8 = de.read_bytes(1)?[0];
        if is_some != 0 {
            Ok(Some(de.read_value::<F, T>(true)?))
        } else {
            Ok(None)
        }
    }

    #[inline(always)]
    fn deserialize_in_place(&mut self, mut de: Deserializer<'de>) -> Result<(), DeserializeError> {
        let is_some: u8 = de.read_bytes(1)?[0];
        if is_some != 0 {
            match self {
                Some(value) => {
                    de.read_in_place::<F, T>(value, true)?;
                }
                None => {
                    *self = Some(de.read_value::<F, T>(true)?);
                }
            }
        } else {
            *self = None;
        }
        Ok(())
    }
}
