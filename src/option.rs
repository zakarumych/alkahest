use crate::{
    deserialize::{Deserializer, Error, NonRefDeserialize},
    formula::{combine_sizes, Formula, NonRefFormula},
    serialize::{NonRefSerializeOwned, Serializer},
};

impl<F> NonRefFormula for Option<F>
where
    F: Formula,
{
    const MAX_SIZE: Option<usize> = combine_sizes(Some(1), F::MAX_SIZE);
}

impl<F, T> NonRefSerializeOwned<Option<F>> for Option<T>
where
    F: Formula,
    T: NonRefSerializeOwned<F::NonRef>,
{
    #[inline(always)]
    fn serialize_owned<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
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

impl<'de, F, T> NonRefDeserialize<'de, Option<F>> for Option<T>
where
    F: Formula,
    T: NonRefDeserialize<'de, F::NonRef>,
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
