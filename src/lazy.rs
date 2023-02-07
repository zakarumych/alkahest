use core::marker::PhantomData;

use crate::{
    deserialize::{Deserialize, Deserializer, Error},
    formula::NonRefFormula,
};

/// Wrapper for lazy deserialization.
/// `Lazy<F>` may deserialize data from formula `F`.
/// Then any it may produce any type `T` that can be deserialized from formula `F`.
#[derive(Clone)]
pub struct Lazy<'de, F: ?Sized> {
    de: Deserializer<'de>,
    marker: PhantomData<fn(&F) -> &F>,
}

impl<'de, F> Lazy<'de, F>
where
    F: NonRefFormula + ?Sized,
{
    /// Deserialize the lazy value.
    #[inline(always)]
    pub fn get<T>(&self) -> Result<T, Error>
    where
        T: Deserialize<'de, F>,
    {
        <T as Deserialize<'de, F>>::deserialize(self.de.clone())
    }

    /// Deserialize the lazy value in place.
    #[inline(always)]
    pub fn get_in_place<T>(&self, place: &mut T) -> Result<(), Error>
    where
        T: Deserialize<'de, F> + ?Sized,
    {
        <T as Deserialize<'de, F>>::deserialize_in_place(place, self.de.clone())
    }
}

impl<'de, 'fe: 'de, F> Deserialize<'fe, F> for Lazy<'de, F>
where
    F: NonRefFormula + ?Sized,
{
    #[inline(always)]
    fn deserialize(de: Deserializer<'fe>) -> Result<Self, Error> {
        Ok(Lazy {
            de,
            marker: PhantomData,
        })
    }

    #[inline(always)]
    fn deserialize_in_place(&mut self, de: Deserializer<'fe>) -> Result<(), Error> {
        self.de = de;
        Ok(())
    }
}
