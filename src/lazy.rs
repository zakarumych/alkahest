use crate::{
    deserialize::{Deserialize, Deserializer, Error},
    formula::NonRefFormula,
};

/// Wrapper for lazy deserialization.
/// `Lazy<T>` may deserialize data from formula `F`
/// when and only when `T` can deserialize data from formula `F`.
/// The actual deserialization is delayed until `get` or `get_in_place` is called.
#[derive(Clone)]
pub struct Lazy<'de, T> {
    de: Deserializer<'de>,
    value: fn(Deserializer<'de>) -> Result<T, Error>,
    in_place: fn(&mut T, Deserializer<'de>) -> Result<(), Error>,
}

impl<'de, T> Lazy<'de, T> {
    /// Deserialize the lazy value.
    #[inline(always)]
    pub fn get(&self) -> Result<T, Error> {
        (self.value)(self.de.clone())
    }

    /// Deserialize the lazy value in place.
    #[inline(always)]
    pub fn get_in_place(&self, place: &mut T) -> Result<(), Error> {
        (self.in_place)(place, self.de.clone())
    }
}

impl<'de, F, T> Deserialize<'de, F> for Lazy<'de, T>
where
    F: NonRefFormula + ?Sized,
    T: Deserialize<'de, F>,
{
    #[inline(always)]
    fn deserialize(de: Deserializer<'de>) -> Result<Self, Error> {
        Ok(Lazy {
            de,
            value: T::deserialize,
            in_place: T::deserialize_in_place,
        })
    }

    #[inline(always)]
    fn deserialize_in_place(&mut self, de: Deserializer<'de>) -> Result<(), Error> {
        self.de = de;
        self.value = T::deserialize;
        self.in_place = T::deserialize_in_place;
        Ok(())
    }
}
