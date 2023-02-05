use crate::{
    deserialize::{Deserialize, Deserializer, Error},
    formula::{combine_sizes, Formula, NonRefFormula},
    serialize::{Serialize, Serializer},
};

impl<F> Formula for Option<F>
where
    F: Formula,
{
    const MAX_SIZE: Option<usize> = combine_sizes(Some(1), F::MAX_SIZE);
}
impl<T> NonRefFormula for Option<T> where T: Formula {}

impl<T, U> Serialize<Option<T>> for Option<U>
where
    T: Formula,
    U: Serialize<T>,
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
                ser.write_value(value)?;
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
            Ok(Some(de.read_value()?))
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
                    *self = Some(de.read_value()?);
                }
            }
        } else {
            *self = None;
        }
        Ok(())
    }
}
