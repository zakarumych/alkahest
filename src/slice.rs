use core::iter::FusedIterator;

use crate::{
    deserialize::{DeIter, Deserialize, Deserializer, Error},
    formula::{Formula, NonRefFormula},
    serialize::{Serialize, Serializer},
};

impl<F> NonRefFormula for [F]
where
    F: Formula,
{
    const MAX_SIZE: Option<usize> = None;
}

impl<F, T, I> Serialize<[F]> for I
where
    F: Formula,
    I: IntoIterator<Item = T>,
    T: Serialize<F>,
{
    #[inline(always)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut ser = ser.into();
        for elem in self {
            ser.write_value::<F, T>(elem)?;
        }
        ser.finish()
    }
}

pub struct LazySlice<'de, F: ?Sized, T = F> {
    inner: DeIter<'de, F, T>,
}

impl<'de, F, T> LazySlice<'de, F, T>
where
    F: ?Sized,
{
    #[inline(always)]
    pub fn iter(&self) -> SliceIter<'de, F, T>
    where
        F: Formula,
        T: Deserialize<'de, F>,
    {
        SliceIter {
            inner: self.inner.clone(),
        }
    }
}

impl<'de, F, T> IntoIterator for LazySlice<'de, F, T>
where
    F: Formula + ?Sized,
    T: Deserialize<'de, F>,
{
    type Item = Result<T, Error>;
    type IntoIter = SliceIter<'de, F, T>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        SliceIter { inner: self.inner }
    }
}

pub struct SliceIter<'de, F: ?Sized, T = F> {
    inner: DeIter<'de, F, T>,
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

impl<'de, F, T> Iterator for SliceIter<'de, F, T>
where
    F: Formula + ?Sized,
    T: Deserialize<'de, F>,
{
    type Item = Result<T, Error>;

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }

    #[inline(always)]
    fn next(&mut self) -> Option<Result<T, Error>> {
        self.inner.next()
    }

    #[inline(always)]
    fn count(self) -> usize {
        self.inner.count()
    }

    #[inline(always)]
    fn nth(&mut self, n: usize) -> Option<Result<T, Error>> {
        self.inner.nth(n)
    }

    #[inline(always)]
    fn fold<B, Fun>(self, init: B, f: Fun) -> B
    where
        Fun: FnMut(B, Result<T, Error>) -> B,
    {
        self.inner.fold(init, f)
    }
}

impl<'de, F, T> DoubleEndedIterator for SliceIter<'de, F, T>
where
    F: Formula + ?Sized,
    T: Deserialize<'de, F>,
{
    #[inline(always)]
    fn next_back(&mut self) -> Option<Result<T, Error>> {
        self.inner.next_back()
    }

    #[inline(always)]
    fn nth_back(&mut self, n: usize) -> Option<Result<T, Error>> {
        self.inner.nth_back(n)
    }

    #[inline(always)]
    fn rfold<B, Fun>(self, init: B, f: Fun) -> B
    where
        Fun: FnMut(B, Result<T, Error>) -> B,
    {
        self.inner.rfold(init, f)
    }
}

impl<'de, F, T> ExactSizeIterator for SliceIter<'de, F, T>
where
    F: Formula + ?Sized,
    T: Deserialize<'de, F>,
{
    #[inline(always)]
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<'de, F, T> FusedIterator for SliceIter<'de, F, T>
where
    F: Formula + ?Sized,
    T: Deserialize<'de, F>,
{
}
