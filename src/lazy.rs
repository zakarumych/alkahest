use core::marker::PhantomData;

use crate::{
    deserialize::{Deserialize, Deserializer, Error},
    formula::{unwrap_size, BareFormula},
    DeIter, Formula,
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
    F: BareFormula + ?Sized,
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
    F: BareFormula + ?Sized,
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

pub struct LazySeq<'de, F: ?Sized, T = F> {
    inner: DeIter<'de, F, T>,
}

impl<'de, F, T> LazySeq<'de, F, T>
where
    F: ?Sized,
{
    #[inline(always)]
    pub fn iter(&self) -> DeIter<'de, F, T>
    where
        F: Formula,
        T: Deserialize<'de, F>,
    {
        self.inner.clone()
    }
}

impl<'de, F, T> IntoIterator for LazySeq<'de, F, T>
where
    F: Formula + ?Sized,
    T: Deserialize<'de, F>,
{
    type Item = Result<T, Error>;
    type IntoIter = DeIter<'de, F, T>;

    #[inline(always)]
    fn into_iter(self) -> DeIter<'de, F, T> {
        self.inner
    }
}

impl<'de, 'fe: 'de, F, T> Deserialize<'fe, [F]> for LazySeq<'de, F, T>
where
    F: Formula,
    T: Deserialize<'de, F>,
{
    #[inline(always)]
    fn deserialize(de: Deserializer<'fe>) -> Result<Self, Error> {
        Ok(LazySeq {
            inner: de.into_iter()?,
        })
    }

    #[inline(always)]
    fn deserialize_in_place(&mut self, de: Deserializer<'fe>) -> Result<(), Error> {
        self.inner = de.into_iter()?;
        Ok(())
    }
}

impl<'de, 'fe: 'de, F, T, const N: usize> Deserialize<'fe, [F; N]> for LazySeq<'de, F, T>
where
    F: Formula,
    T: Deserialize<'de, F>,
{
    #[inline(always)]
    fn deserialize(de: Deserializer<'fe>) -> Result<Self, Error> {
        Ok(LazySeq {
            inner: de.into_iter()?,
        })
    }

    #[inline(always)]
    fn deserialize_in_place(&mut self, de: Deserializer<'fe>) -> Result<(), Error> {
        self.inner = de.into_iter()?;
        Ok(())
    }
}

pub struct LazySlice<'de, F: ?Sized, T = F> {
    inner: DeIter<'de, F, T>,
}

impl<'de, F, T> LazySlice<'de, F, T>
where
    F: Formula + ?Sized,
    T: Deserialize<'de, F>,
{
    pub const ELEMENT_SIZE: usize = unwrap_size(F::MAX_STACK_SIZE);
}

impl<'de, F, T> LazySlice<'de, F, T>
where
    F: ?Sized,
{
    #[inline(always)]
    pub fn iter(&self) -> DeIter<'de, F, T>
    where
        F: Formula,
        T: Deserialize<'de, F>,
    {
        self.inner.clone()
    }
}

impl<'de, F, T> IntoIterator for LazySlice<'de, F, T>
where
    F: Formula + ?Sized,
    T: Deserialize<'de, F>,
{
    type Item = Result<T, Error>;
    type IntoIter = DeIter<'de, F, T>;

    #[inline(always)]
    fn into_iter(self) -> DeIter<'de, F, T> {
        self.inner
    }
}

impl<'de, 'fe: 'de, F, T> Deserialize<'fe, [F]> for LazySlice<'de, F, T>
where
    F: Formula,
    T: Deserialize<'de, F>,
{
    #[inline(always)]
    fn deserialize(de: Deserializer<'fe>) -> Result<Self, Error> {
        Ok(LazySlice {
            inner: de.into_iter()?,
        })
    }

    #[inline(always)]
    fn deserialize_in_place(&mut self, de: Deserializer<'fe>) -> Result<(), Error> {
        self.inner = de.into_iter()?;
        Ok(())
    }
}

impl<'de, 'fe: 'de, F, T, const N: usize> Deserialize<'fe, [F; N]> for LazySlice<'de, F, T>
where
    F: Formula,
    T: Deserialize<'de, F>,
{
    #[inline(always)]
    fn deserialize(de: Deserializer<'fe>) -> Result<Self, Error> {
        Ok(LazySlice {
            inner: de.into_iter()?,
        })
    }

    #[inline(always)]
    fn deserialize_in_place(&mut self, de: Deserializer<'fe>) -> Result<(), Error> {
        self.inner = de.into_iter()?;
        Ok(())
    }
}
