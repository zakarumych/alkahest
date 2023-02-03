use crate::{
    deserialize::{Deserialize, DeserializeError},
    formula::NonRefFormula,
};

/// Wrapper for lazy deserialization.
/// `Lazy<T>` may deserialize data from formula `F`
/// when and only when `T` can deserialize data from formula `F`.
/// The actual deserialization is delayed until `get` or `get_in_place` is called.
pub struct Lazy<'de, T> {
    len: usize,
    input: &'de [u8],
    de: fn(usize, &'de [u8]) -> Result<T, DeserializeError>,
    de_in_place: fn(&mut T, usize, &'de [u8]) -> Result<(), DeserializeError>,
}

impl<'de, T> Lazy<'de, T> {
    /// Deserialize the lazy value.
    pub fn get(self) -> Result<T, DeserializeError> {
        (self.de)(self.len, self.input)
    }

    /// Deserialize the lazy value in place.
    pub fn get_in_place(self, place: &mut T) -> Result<(), DeserializeError> {
        (self.de_in_place)(place, self.len, self.input)
    }
}

impl<'de, F, T> Deserialize<'de, F> for Lazy<'de, T>
where
    F: NonRefFormula + ?Sized,
    T: Deserialize<'de, F>,
{
    fn deserialize(len: usize, input: &'de [u8]) -> Result<Self, DeserializeError> {
        Ok(Lazy {
            len,
            input,
            de: T::deserialize,
            de_in_place: T::deserialize_in_place,
        })
    }

    fn deserialize_in_place(
        &mut self,
        len: usize,
        input: &'de [u8],
    ) -> Result<(), DeserializeError> {
        *self = <Self as Deserialize<'de, F>>::deserialize(len, input)?;
        Ok(())
    }
}
