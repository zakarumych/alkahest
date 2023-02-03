use crate::{
    deserialize::{Deserialize, DeserializeError, Deserializer},
    formula::{Formula, NonRefFormula, UnsizedFormula},
    serialize::{Serialize, Serializer},
};

impl<T> UnsizedFormula for Option<T> where T: Formula {}
impl<T> Formula for Option<T>
where
    T: Formula,
{
    const SIZE: usize = 1 + T::SIZE;
}
impl<T> NonRefFormula for Option<T> where T: Formula {}

impl<T, U> Serialize<Option<T>> for Option<U>
where
    T: Formula,
    U: Serialize<T>,
{
    fn serialize(self, offset: usize, output: &mut [u8]) -> Result<(usize, usize), usize> {
        let mut ser = Serializer::new(offset, output);
        match self {
            Some(value) => {
                ser.serialize_self::<u8>(1)?;
                ser.serialize_unsized(value)?;
                Ok(ser.finish())
            }
            None => {
                ser.serialize_self::<u8>(0)?;
                ser.waste(T::SIZE)?;
                Ok(ser.finish())
            }
        }
    }
}

impl<'de, F, T> Deserialize<'de, Option<F>> for Option<T>
where
    F: Formula,
    T: Deserialize<'de, F>,
{
    fn deserialize(len: usize, input: &'de [u8]) -> Result<Self, DeserializeError> {
        let mut de = Deserializer::new(len, input)?;
        let is_some = de.deserialize_self::<u8>()?;
        if is_some != 0 {
            Ok(Some(de.deserialize_sized()?))
        } else {
            de.consume(F::SIZE)?;
            Ok(None)
        }
    }

    fn deserialize_in_place(
        &mut self,
        len: usize,
        input: &'de [u8],
    ) -> Result<(), DeserializeError> {
        let mut de = Deserializer::new(len, input)?;
        let is_some = de.deserialize_self::<u8>()?;
        if is_some != 0 {
            match self {
                Some(value) => {
                    de.deserialize_in_place_sized::<F, T>(value)?;
                }
                None => {
                    *self = Some(de.deserialize_sized()?);
                }
            }
        } else {
            de.consume(F::SIZE)?;
            *self = None;
        }
        Ok(())
    }
}
