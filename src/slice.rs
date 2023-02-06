use core::{iter::FusedIterator, ops::Range};

use crate::{
    deserialize::{DeIter, Deserialize, Deserializer, Error},
    formula::{Formula, NonRefFormula},
    serialize::{SerializeOwned, Serializer},
};

impl<F> NonRefFormula for [F]
where
    F: Formula,
{
    const MAX_SIZE: Option<usize> = None;
}

/// Wrapper for iterators to implement `SerializeOwned` into slice formula.
#[repr(transparent)]
pub struct SerIter<I>(pub I);

impl<F, T, I> SerializeOwned<[F]> for SerIter<I>
where
    F: Formula,
    I: IntoIterator<Item = T>,
    T: SerializeOwned<F::NonRef>,
{
    #[cfg_attr(feature = "inline-more", inline(always))]
    fn serialize_owned<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
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

impl<F, T> SerializeOwned<[F]> for Range<T>
where
    Range<T>: IntoIterator<Item = T>,
    T: SerializeOwned<F::NonRef>,
    F: Formula,
{
    fn serialize_owned<S>(self, er: impl Into<S>) -> Result<S::Ok, S::Error>
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
    T: Deserialize<'de, F::NonRef>,
{
    #[cfg_attr(feature = "inline-more", inline(always))]
    fn deserialize(de: Deserializer<'de>) -> Result<Self, Error> {
        Ok(SliceIter {
            inner: de.into_iter()?,
        })
    }

    #[cfg_attr(feature = "inline-more", inline(always))]
    fn deserialize_in_place(&mut self, de: Deserializer<'de>) -> Result<(), Error> {
        self.inner = de.into_iter()?;
        Ok(())
    }
}

impl<'de, F, T, const N: usize> Deserialize<'de, [F; N]> for SliceIter<'de, F, T>
where
    F: Formula,
    T: Deserialize<'de, F::NonRef>,
{
    #[cfg_attr(feature = "inline-more", inline(always))]
    fn deserialize(de: Deserializer<'de>) -> Result<Self, Error> {
        Ok(SliceIter {
            inner: de.into_iter()?,
        })
    }

    #[cfg_attr(feature = "inline-more", inline(always))]
    fn deserialize_in_place(&mut self, de: Deserializer<'de>) -> Result<(), Error> {
        self.inner = de.into_iter()?;
        Ok(())
    }
}

impl<'de, F, T> Iterator for SliceIter<'de, F, T>
where
    F: Formula,
    T: Deserialize<'de, F::NonRef>,
{
    type Item = Result<T, Error>;

    #[cfg_attr(feature = "inline-more", inline(always))]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }

    #[cfg_attr(feature = "inline-more", inline(always))]
    fn next(&mut self) -> Option<Result<T, Error>> {
        self.inner.next()
    }

    #[cfg_attr(feature = "inline-more", inline(always))]
    fn count(self) -> usize {
        self.inner.count()
    }

    #[cfg_attr(feature = "inline-more", inline(always))]
    fn nth(&mut self, n: usize) -> Option<Result<T, Error>> {
        self.inner.nth(n)
    }

    #[cfg_attr(feature = "inline-more", inline(always))]
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
    T: Deserialize<'de, F::NonRef>,
{
    #[cfg_attr(feature = "inline-more", inline(always))]
    fn next_back(&mut self) -> Option<Result<T, Error>> {
        self.inner.next_back()
    }

    #[cfg_attr(feature = "inline-more", inline(always))]
    fn nth_back(&mut self, n: usize) -> Option<Result<T, Error>> {
        self.inner.nth_back(n)
    }

    #[cfg_attr(feature = "inline-more", inline(always))]
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
    T: Deserialize<'de, F::NonRef>,
{
    #[cfg_attr(feature = "inline-more", inline(always))]
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<'de, F, T> FusedIterator for SliceIter<'de, F, T>
where
    F: Formula,
    T: Deserialize<'de, F::NonRef>,
{
}
