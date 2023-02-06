use core::{iter::FusedIterator, ops::Range};

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

/// Wrapper for iterators to implement `Serialize` into slice formula.
#[repr(transparent)]
pub struct SerIter<I>(pub I);

impl<F, T, I> Serialize<[F]> for SerIter<I>
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
        for elem in self.0.into_iter() {
            ser.write_value::<F, T>(elem)?;
        }
        ser.finish()
    }
}

impl<F, T> Serialize<[F]> for Range<T>
where
    Range<T>: IntoIterator<Item = T>,
    T: Serialize<F>,
    F: Formula,
{
    fn serialize<S>(self, er: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut er = er.into();
        for elem in self {
            er.write_value::<F, _>(elem)?;
        }
        er.finish()
    }
}

pub struct SliceIter<'de, F, T = F> {
    inner: DeIter<'de, F, T>,
}

impl<'de, F, T> Deserialize<'de, [F]> for SliceIter<'de, F, T>
where
    F: Formula,
    T: Deserialize<'de, F>,
{
    #[inline(always)]
    fn deserialize(de: Deserializer<'de>) -> Result<Self, Error> {
        Ok(SliceIter {
            inner: de.into_iter()?,
        })
    }

    #[inline(always)]
    fn deserialize_in_place(&mut self, de: Deserializer<'de>) -> Result<(), Error> {
        self.inner = de.into_iter()?;
        Ok(())
    }
}

impl<'de, F, T, const N: usize> Deserialize<'de, [F; N]> for SliceIter<'de, F, T>
where
    F: Formula,
    T: Deserialize<'de, F>,
{
    #[inline(always)]
    fn deserialize(de: Deserializer<'de>) -> Result<Self, Error> {
        Ok(SliceIter {
            inner: de.into_iter()?,
        })
    }

    #[inline(always)]
    fn deserialize_in_place(&mut self, de: Deserializer<'de>) -> Result<(), Error> {
        self.inner = de.into_iter()?;
        Ok(())
    }
}

impl<'de, F, T> Iterator for SliceIter<'de, F, T>
where
    F: Formula,
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
    F: Formula,
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
    F: Formula,
    T: Deserialize<'de, F>,
{
    #[inline(always)]
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<'de, F, T> FusedIterator for SliceIter<'de, F, T>
where
    F: Formula,
    T: Deserialize<'de, F>,
{
}
