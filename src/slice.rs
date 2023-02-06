use core::iter::FusedIterator;

use crate::{
    deserialize::{DeIter, Deserializer, Error, NonRefDeserialize},
    formula::{Formula, NonRefFormula},
    serialize::{NonRefSerializeOwned, SerializeOwned, Serializer},
};

impl<F> NonRefFormula for [F]
where
    F: Formula,
{
    const MAX_SIZE: Option<usize> = None;
}

/// Wrapper for iterators to implement `NonRefSerializeOwned` into slice formula.
#[repr(transparent)]
pub struct SerIter<I>(pub I);

impl<F, T, I> NonRefSerializeOwned<[F]> for SerIter<I>
where
    F: Formula,
    I: IntoIterator<Item = T>,
    T: SerializeOwned<F>,
{
    #[inline(always)]
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
macro_rules! impl_iter_to_slice {
    (for<F $(,$a:ident)*> $iter:ty where $($predicates:tt)*) => {
        impl<F $(, $a)*> NonRefSerializeOwned<[F]> for $iter
        where $($predicates)*
        {
            #[inline(always)]
            fn serialize_owned<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                let mut ser = ser.into();
                for elem in self {
                    ser.write_value::<F, _>(elem)?;
                }
                ser.finish()
            }
        }
    };
}

impl_iter_to_slice!(for<F, T> core::ops::Range<T> where F: Formula, T: SerializeOwned<F>, core::ops::Range<T>: IntoIterator<Item = T>);

#[cfg(feature = "alloc")]
impl_iter_to_slice!(for<F, T> alloc::vec::Vec<T> where F: Formula, T: SerializeOwned<F>);

#[cfg(feature = "alloc")]
impl_iter_to_slice!(for<F, T> alloc::collections::VecDeque<T> where F: Formula, T: SerializeOwned<F>);

pub struct SliceIter<'de, F, T = F> {
    inner: DeIter<'de, F, T>,
}

impl<'de, F, T> NonRefDeserialize<'de, [F]> for SliceIter<'de, F, T>
where
    F: Formula,
    T: NonRefDeserialize<'de, F::NonRef>,
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

impl<'de, F, T, const N: usize> NonRefDeserialize<'de, [F; N]> for SliceIter<'de, F, T>
where
    F: Formula,
    T: NonRefDeserialize<'de, F::NonRef>,
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
    T: NonRefDeserialize<'de, F::NonRef>,
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
    T: NonRefDeserialize<'de, F::NonRef>,
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
    T: NonRefDeserialize<'de, F::NonRef>,
{
    #[inline(always)]
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<'de, F, T> FusedIterator for SliceIter<'de, F, T>
where
    F: Formula,
    T: NonRefDeserialize<'de, F::NonRef>,
{
}
