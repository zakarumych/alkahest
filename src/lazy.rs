use core::{
    any::type_name,
    fmt::{self, Debug},
    marker::PhantomData,
};

use crate::{
    deserialize::{DeIter, Deserialize, DeserializeError, Deserializer, SizedDeIter},
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

impl<'de, F> Debug for Lazy<'de, F>
where
    F: ?Sized,
{
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Lazy<{:?}>", type_name::<F>())
    }
}

impl<'de, F> Lazy<'de, F>
where
    F: BareFormula + ?Sized,
{
    /// Deserialize the lazy value.
    #[inline(always)]
    pub fn get<T>(&self) -> Result<T, DeserializeError>
    where
        T: Deserialize<'de, F>,
    {
        <T as Deserialize<'de, F>>::deserialize(self.de.clone())
    }

    /// Deserialize the lazy value in place.
    #[inline(always)]
    pub fn get_in_place<T>(&self, place: &mut T) -> Result<(), DeserializeError>
    where
        T: Deserialize<'de, F> + ?Sized,
    {
        <T as Deserialize<'de, F>>::deserialize_in_place(place, self.de.clone())
    }
}

trait LazySizedIter<'de, F: ?Sized> {
    const ELEMENT_SIZE: usize;

    fn sized_iter_impl<T>(&self) -> SizedDeIter<'de, F, T>
    where
        F: Formula,
        T: Deserialize<'de, F>;
}

impl<'de, F> LazySizedIter<'de, F> for Lazy<'de, [F]>
where
    F: Formula,
{
    // Fail compilation.
    // Use `Lazy::iter` instead of `Lazy::sized_iter` for unsized formulas.
    const ELEMENT_SIZE: usize = unwrap_size(F::MAX_STACK_SIZE);

    #[inline(always)]
    fn sized_iter_impl<T>(&self) -> SizedDeIter<'de, F, T>
    where
        T: Deserialize<'de, F>,
    {
        assert_eq!(Some(Self::ELEMENT_SIZE), F::MAX_STACK_SIZE);
        self.de.clone().into_sized_iter()
    }
}

impl<'de, F> Lazy<'de, [F]>
where
    F: Formula,
{
    /// Produce iterator over lazy deserialized values.
    /// # Example
    ///
    /// ```
    /// # use alkahest::*;
    /// let mut buffer = [0u8; 1024];
    ///
    /// serialize::<[u32], _>([1, 2, 3], &mut buffer).unwrap();
    /// let (lazy, _) = deserialize::<[u32], Lazy<[u32]>>(&buffer).unwrap();
    /// let mut iter = lazy.sized_iter::<u32>();
    /// assert_eq!(iter.next().unwrap().unwrap(), 1);
    /// assert_eq!(iter.next().unwrap().unwrap(), 2);
    /// assert_eq!(iter.next().unwrap().unwrap(), 3);
    /// assert!(iter.next().is_none());
    /// ```
    ///
    /// `sized_iter` cannot be used to deserialize slice of unsized formulas.
    /// Attempt to use unsized formula will result in compile error.
    ///
    /// ```compile_fail
    /// # use alkahest::*;
    /// let mut buffer = [0u8; 1024];
    ///
    /// serialize::<[As<str>], _>(["qwe", "rty"], &mut buffer).unwrap();
    /// let (lazy, _) = deserialize::<[As<str>], Lazy<[As<str>]>>(&buffer).unwrap();
    /// let mut iter = lazy.sized_iter::<&str>();
    /// assert_eq!(iter.next().unwrap().unwrap(), "qwe");
    /// assert_eq!(iter.next().unwrap().unwrap(), "rty");
    /// assert!(iter.next().is_none());
    /// ```
    #[inline(always)]
    pub fn sized_iter<T>(&self) -> SizedDeIter<'de, F, T>
    where
        T: Deserialize<'de, F>,
    {
        self.sized_iter_impl()
    }

    /// Produce iterator over lazy deserialized values.
    ///
    /// # Example
    ///
    /// ```
    /// # use alkahest::*;
    /// let mut buffer = [0u8; 1024];
    ///
    /// serialize::<[u32], _>([1, 2, 3], &mut buffer).unwrap();
    /// let (lazy, _) = deserialize::<[u32], Lazy<[u32]>>(&buffer).unwrap();
    /// let mut iter = lazy.iter::<u32>();
    /// assert_eq!(iter.next().unwrap().unwrap(), 1);
    /// assert_eq!(iter.next().unwrap().unwrap(), 2);
    /// assert_eq!(iter.next().unwrap().unwrap(), 3);
    /// assert!(iter.next().is_none());
    /// ```
    ///
    /// `iter` may be used to deserialize slice of unsized formulas.
    ///
    /// ```
    /// # use alkahest::*;
    /// let mut buffer = [0u8; 1024];
    ///
    /// serialize::<[As<str>], _>(["qwe", "rty"], &mut buffer).unwrap();
    /// let (seq, _) = deserialize::<[As<str>], Lazy<[As<str>]>>(&buffer).unwrap();
    /// let mut iter = seq.iter::<&str>();
    /// assert_eq!(iter.next().unwrap().unwrap(), "qwe");
    /// assert_eq!(iter.next().unwrap().unwrap(), "rty");
    /// assert!(iter.next().is_none());
    /// ```
    #[inline(always)]
    pub fn iter<T>(&self) -> DeIter<'de, F, T>
    where
        T: Deserialize<'de, F>,
    {
        self.de.clone().into_unsized_iter()
    }
}

impl<'de, 'fe: 'de, F> Deserialize<'fe, F> for Lazy<'de, F>
where
    F: BareFormula + ?Sized,
{
    #[inline(always)]
    fn deserialize(de: Deserializer<'fe>) -> Result<Self, DeserializeError> {
        Ok(Lazy {
            de,
            marker: PhantomData,
        })
    }

    #[inline(always)]
    fn deserialize_in_place(&mut self, de: Deserializer<'fe>) -> Result<(), DeserializeError> {
        self.de = de;
        Ok(())
    }
}
