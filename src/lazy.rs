use core::marker::PhantomData;

use crate::{
    deserialize::{DeIter, Deserialize, Deserializer, Error, UnsizedDeIter},
    formula::{unwrap_size, BareFormula, Formula},
};

/// Wrapper for lazy deserialization.
/// `Lazy<F>` may deserialize data from formula `F`.
/// Then any it may produce any type `T` that can be deserialized from formula `F`. ```
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
    #[inline(never)]
    pub fn get<T>(&self) -> Result<T, Error>
    where
        T: Deserialize<'de, F>,
    {
        <T as Deserialize<'de, F>>::deserialize(self.de.clone())
    }

    /// Deserialize the lazy value in place.
    #[inline(never)]
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
    #[inline(never)]
    fn deserialize(de: Deserializer<'fe>) -> Result<Self, Error> {
        Ok(Lazy {
            de,
            marker: PhantomData,
        })
    }

    #[inline(never)]
    fn deserialize_in_place(&mut self, de: Deserializer<'fe>) -> Result<(), Error> {
        self.de = de;
        Ok(())
    }
}

/// Wrapper for lazy deserialization of a sequence.
/// `LazySeq<F, T>` may deserialize data from formula `[F]`
/// where `T` can be deserialized from formula `F`.
///
/// # Example
///
/// ```
/// # use alkahest::*;
/// let mut buffer = [0u8; 1024];
///
/// serialize::<[u32], _>([1, 2, 3], &mut buffer).unwrap();
/// let (seq, _) = deserialize::<[u32], LazySeq<u32, u32>>(&buffer).unwrap();
/// let mut iter = seq.iter();
/// assert_eq!(iter.next().unwrap().unwrap(), 1);
/// assert_eq!(iter.next().unwrap().unwrap(), 2);
/// assert_eq!(iter.next().unwrap().unwrap(), 3);
/// assert!(iter.next().is_none());
/// ```
///
/// `LazySeq` may be used to deserialize slice of unsized formulas.
///
/// ```
/// # use alkahest::*;
/// let mut buffer = [0u8; 1024];
///
/// serialize::<[As<str>], _>(["qwe", "rty"], &mut buffer).unwrap();
/// let (seq, _) = deserialize::<[As<str>], LazySeq<As<str>, &str>>(&buffer).unwrap();
/// let mut iter = seq.iter();
/// assert_eq!(iter.next().unwrap().unwrap(), "qwe");
/// assert_eq!(iter.next().unwrap().unwrap(), "rty");
/// assert!(iter.next().is_none());
/// ```
pub struct LazySeq<'de, F: ?Sized, T = F> {
    inner: UnsizedDeIter<'de, F, T>,
}

impl<'de, F, T> LazySeq<'de, F, T>
where
    F: Formula + ?Sized,
    T: Deserialize<'de, F>,
{
    #[inline(never)]
    pub fn iter(&self) -> UnsizedDeIter<'de, F, T> {
        self.inner.clone()
    }
}

impl<'de, F, T> IntoIterator for LazySeq<'de, F, T>
where
    F: Formula + ?Sized,
    T: Deserialize<'de, F>,
{
    type Item = Result<T, Error>;
    type IntoIter = UnsizedDeIter<'de, F, T>;

    #[inline(never)]
    fn into_iter(self) -> UnsizedDeIter<'de, F, T> {
        self.inner
    }
}

impl<'de, 'fe: 'de, F, T> Deserialize<'fe, [F]> for LazySeq<'de, F, T>
where
    F: Formula,
    T: Deserialize<'de, F>,
{
    #[inline(never)]
    fn deserialize(de: Deserializer<'fe>) -> Result<Self, Error> {
        Ok(LazySeq {
            inner: de.into_unsized_iter()?,
        })
    }

    #[inline(never)]
    fn deserialize_in_place(&mut self, de: Deserializer<'fe>) -> Result<(), Error> {
        self.inner = de.into_unsized_iter()?;
        Ok(())
    }
}

impl<'de, 'fe: 'de, F, T, const N: usize> Deserialize<'fe, [F; N]> for LazySeq<'de, F, T>
where
    F: Formula,
    T: Deserialize<'de, F>,
{
    #[inline(never)]
    fn deserialize(de: Deserializer<'fe>) -> Result<Self, Error> {
        Ok(LazySeq {
            inner: de.into_unsized_iter()?,
        })
    }

    #[inline(never)]
    fn deserialize_in_place(&mut self, de: Deserializer<'fe>) -> Result<(), Error> {
        self.inner = de.into_unsized_iter()?;
        Ok(())
    }
}

/// Wrapper for lazy deserialization of a sequence.
/// `LazySlice<F, T>` may deserialize data from formula `[F]`
/// where `T` can be deserialized from formula `F`.
///
/// # Example
///
/// ```
/// # use alkahest::*;
/// let mut buffer = [0u8; 1024];
///
/// serialize::<[u32], _>([1, 2, 3], &mut buffer).unwrap();
/// let (seq, _) = deserialize::<[u32], LazySlice<u32, u32>>(&buffer).unwrap();
/// let mut iter = seq.iter();
/// assert_eq!(iter.next().unwrap().unwrap(), 1);
/// assert_eq!(iter.next().unwrap().unwrap(), 2);
/// assert_eq!(iter.next().unwrap().unwrap(), 3);
/// assert!(iter.next().is_none());
/// ```
///
/// `LazySlice` cannot be used to deserialize slice of unsized formulas.
/// Attempt to use unsized formula will result in compile error.
///
/// ```compile_fail
/// # use alkahest::*;
/// let mut buffer = [0u8; 1024];
///
/// serialize::<[As<str>], _>(["qwe", "rty"], &mut buffer).unwrap();
/// let (seq, _) = deserialize::<[As<str>], LazySlice<As<str>, &str>>(&buffer).unwrap();
/// let mut iter = seq.iter();
/// assert_eq!(iter.next().unwrap().unwrap(), "qwe");
/// assert_eq!(iter.next().unwrap().unwrap(), "rty");
/// assert!(iter.next().is_none());
/// ```
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
    F: Formula + ?Sized,
    T: Deserialize<'de, F>,
{
    #[inline(never)]
    pub fn iter(&self) -> DeIter<'de, F, T> {
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

    #[inline(never)]
    fn into_iter(self) -> DeIter<'de, F, T> {
        self.inner
    }
}

impl<'de, 'fe: 'de, F, T> Deserialize<'fe, [F]> for LazySlice<'de, F, T>
where
    F: Formula,
    T: Deserialize<'de, F>,
{
    #[inline(never)]
    fn deserialize(de: Deserializer<'fe>) -> Result<Self, Error> {
        let _ = Self::ELEMENT_SIZE;
        Ok(LazySlice {
            inner: de.into_iter()?,
        })
    }

    #[inline(never)]
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
    #[inline(never)]
    fn deserialize(de: Deserializer<'fe>) -> Result<Self, Error> {
        Ok(LazySlice {
            inner: de.into_iter()?,
        })
    }

    #[inline(never)]
    fn deserialize_in_place(&mut self, de: Deserializer<'fe>) -> Result<(), Error> {
        self.inner = de.into_iter()?;
        Ok(())
    }
}
