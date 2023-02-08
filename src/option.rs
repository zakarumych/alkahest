use crate::{
    deserialize::{Deserialize, Deserializer, Error},
    formula::{sum_size, Formula, NonRefFormula},
    serialize::{Serialize, Serializer},
};

impl<F> Formula for Option<F>
where
    F: Formula,
{
    const MAX_STACK_SIZE: Option<usize> = sum_size(Some(1), F::MAX_STACK_SIZE);
    const EXACT_SIZE: bool = F::EXACT_SIZE;
}

impl<F> NonRefFormula for Option<F> where F: Formula {}

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
            }
            Some(value) => {
                ser.write_bytes(&[1u8])?;
                ser.write_value::<F, T>(value)?;
            }
        }
        ser.finish()
    }
}

impl<F, T> Serialize<Option<F>> for &Option<T>
where
    F: Formula,
    for<'a> &'a T: Serialize<F>,
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
            }
            Some(value) => {
                ser.write_bytes(&[1u8])?;
                ser.write_value::<F, &T>(value)?;
            }
        }
        ser.finish()
    }
}

impl<'de, F, T> Deserialize<'de, Option<F>> for Option<T>
where
    F: Formula,
    T: Deserialize<'de, F>,
{
    #[inline(always)]
    fn deserialize(mut de: Deserializer<'de>) -> Result<Self, Error> {
        let is_some: u8 = de.read_bytes(1)?[0];
        if is_some != 0 {
            Ok(Some(de.read_value::<F, T>()?))
        } else {
            Ok(None)
        }
    }

    #[inline(always)]
    fn deserialize_in_place(&mut self, mut de: Deserializer<'de>) -> Result<(), Error> {
        let is_some: u8 = de.read_bytes(1)?[0];
        if is_some != 0 {
            match self {
                Some(value) => {
                    de.read_in_place::<F, T>(value)?;
                }
                None => {
                    *self = Some(de.read_value::<F, T>()?);
                }
            }
        } else {
            *self = None;
        }
        Ok(())
    }
}
