use crate::{
    deserialize::{Deserialize, DeserializeError, Deserializer},
    formula::Formula,
    serialize::{Serialize, Serializer},
    size::FixedUsize,
    slice::{default_iter_fast_sizes, default_iter_fast_sizes_unchecked},
};

macro_rules! serialize_iter_to_slice {
    ($F:ty : $self:expr => $ser:expr) => {{
        let mut ser = $ser.into();
        ser.write_slice::<$F, _>($self)?;
        ser.finish()
    }};
}

/// Iterator wrapper serializable with slice formula.
/// Many standard library iterators implement serialization.
/// For others this wrapper can be used without performance penalty.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct SerIter<T>(pub T);

impl<F, T, I> Serialize<[F]> for SerIter<I>
where
    F: Formula,
    I: Iterator<Item = T>,
    T: Serialize<F>,
{
    #[inline(always)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_iter_to_slice!(F : self.0 => ser)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<usize> {
        default_iter_fast_sizes::<F, I>(&self.0)
    }
}

impl<F, T> Serialize<[F]> for core::ops::Range<T>
where
    F: Formula,
    T: Serialize<F>,
    core::ops::Range<T>: Iterator<Item = T>,
{
    #[inline(always)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_iter_to_slice!(F : self => ser)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<usize> {
        default_iter_fast_sizes_unchecked::<F, _>(self)
    }
}

impl<F, T> Serialize<[F]> for core::ops::RangeInclusive<T>
where
    F: Formula,
    T: Serialize<F>,
    core::ops::RangeInclusive<T>: Iterator<Item = T>,
{
    #[inline(always)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_iter_to_slice!(F : self => ser)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<usize> {
        default_iter_fast_sizes_unchecked::<F, _>(self)
    }
}

impl<F, A, B, T> Serialize<[F]> for core::iter::Chain<A, B>
where
    F: Formula,
    A: Iterator<Item = T>,
    B: Iterator<Item = T>,
    T: Serialize<F>,
{
    #[inline(always)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_iter_to_slice!(F : self => ser)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<usize> {
        default_iter_fast_sizes::<F, _>(self)
    }
}

impl<'a, F, I, T> Serialize<[F]> for core::iter::Cloned<I>
where
    F: Formula,
    I: Iterator<Item = &'a T>,
    T: Clone + Serialize<F> + 'a,
{
    #[inline(always)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_iter_to_slice!(F : self => ser)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<usize> {
        default_iter_fast_sizes::<F, _>(self)
    }
}

impl<'a, F, I, T> Serialize<[F]> for core::iter::Copied<I>
where
    F: Formula,
    I: Iterator<Item = &'a T>,
    T: Copy + Serialize<F> + 'a,
{
    #[inline(always)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_iter_to_slice!(F : self => ser)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<usize> {
        default_iter_fast_sizes::<F, _>(self)
    }
}

impl<F, T> Serialize<[F]> for core::iter::Empty<T>
where
    F: Formula,
    T: Copy + Serialize<F>,
{
    #[inline(always)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_iter_to_slice!(F : self => ser)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<usize> {
        Some(0)
    }
}

// Typically `usize` is not serializable.
// But lib makes exception for `usize`s that are derived from actual sizes.
impl<'a, F, I, T> Serialize<[(FixedUsize, F)]> for core::iter::Enumerate<I>
where
    F: Formula,
    I: Iterator<Item = T>,
    T: Serialize<F>,
{
    #[inline(always)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_iter_to_slice!((FixedUsize, F) : self => ser)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<usize> {
        default_iter_fast_sizes_unchecked::<(FixedUsize, F), _>(self)
    }
}

impl<F, I, T, P> Serialize<[F]> for core::iter::Filter<I, P>
where
    F: Formula,
    I: Iterator<Item = T>,
    P: FnMut(&T) -> bool,
    T: Serialize<F>,
{
    #[inline(always)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_iter_to_slice!(F : self => ser)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<usize> {
        default_iter_fast_sizes::<F, _>(self)
    }
}

impl<F, I, T, P> Serialize<[F]> for core::iter::FilterMap<I, P>
where
    F: Formula,
    I: Iterator,
    P: FnMut(I::Item) -> Option<T>,
    T: Serialize<F>,
{
    #[inline(always)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_iter_to_slice!(F : self => ser)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<usize> {
        default_iter_fast_sizes::<F, _>(self)
    }
}

impl<F, I, X, U, T> Serialize<[F]> for core::iter::FlatMap<I, U, X>
where
    F: Formula,
    I: Iterator,
    X: FnMut(I::Item) -> U,
    U: IntoIterator<Item = T>,
    T: Serialize<F>,
{
    #[inline(always)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_iter_to_slice!(F : self => ser)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<usize> {
        default_iter_fast_sizes::<F, _>(self)
    }
}

impl<F, I, T> Serialize<[F]> for core::iter::Flatten<I>
where
    F: Formula,
    I: Iterator,
    I::Item: IntoIterator<Item = T>,
    T: Serialize<F>,
{
    #[inline(always)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_iter_to_slice!(F : self => ser)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<usize> {
        default_iter_fast_sizes::<F, _>(self)
    }
}

impl<F, P, T> Serialize<[F]> for core::iter::FromFn<P>
where
    F: Formula,
    P: FnMut() -> Option<T>,
    T: Serialize<F>,
{
    #[inline(always)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_iter_to_slice!(F : self => ser)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<usize> {
        default_iter_fast_sizes::<F, _>(self)
    }
}

impl<F, I, T> Serialize<[F]> for core::iter::Fuse<I>
where
    F: Formula,
    I: Iterator<Item = T>,
    T: Serialize<F>,
{
    #[inline(always)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_iter_to_slice!(F : self => ser)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<usize> {
        default_iter_fast_sizes::<F, _>(self)
    }
}

impl<F, I, T, X> Serialize<[F]> for core::iter::Inspect<I, X>
where
    F: Formula,
    I: Iterator<Item = T>,
    T: Serialize<F>,
    X: FnMut(&T),
{
    #[inline(always)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_iter_to_slice!(F : self => ser)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<usize> {
        default_iter_fast_sizes::<F, _>(self)
    }
}

impl<F, I, T, P> Serialize<[F]> for core::iter::Map<I, P>
where
    F: Formula,
    I: Iterator,
    P: FnMut(I::Item) -> T,
    T: Serialize<F>,
{
    #[inline(always)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_iter_to_slice!(F : self => ser)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<usize> {
        default_iter_fast_sizes::<F, _>(self)
    }
}

impl<F, I, T, P> Serialize<[F]> for core::iter::MapWhile<I, P>
where
    F: Formula,
    I: Iterator,
    P: FnMut(I::Item) -> Option<T>,
    T: Serialize<F>,
{
    #[inline(always)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_iter_to_slice!(F : self => ser)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<usize> {
        default_iter_fast_sizes::<F, _>(self)
    }
}

impl<F, T> Serialize<[F]> for core::iter::Once<T>
where
    F: Formula,
    T: Serialize<F>,
{
    #[inline(always)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_iter_to_slice!(F : self => ser)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<usize> {
        default_iter_fast_sizes::<F, _>(self)
    }
}

impl<F, T, P> Serialize<[F]> for core::iter::OnceWith<P>
where
    F: Formula,
    P: FnOnce() -> T,
    T: Serialize<F>,
{
    #[inline(always)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_iter_to_slice!(F : self => ser)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<usize> {
        default_iter_fast_sizes::<F, _>(self)
    }
}

impl<F, I, T> Serialize<[F]> for core::iter::Peekable<I>
where
    F: Formula,
    I: Iterator<Item = T>,
    T: Serialize<F>,
{
    #[inline(always)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_iter_to_slice!(F : self => ser)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<usize> {
        default_iter_fast_sizes::<F, _>(self)
    }
}

impl<F, I, T> Serialize<[F]> for core::iter::Rev<I>
where
    F: Formula,
    I: DoubleEndedIterator<Item = T>,
    T: Serialize<F>,
{
    #[inline(always)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_iter_to_slice!(F : self => ser)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<usize> {
        default_iter_fast_sizes::<F, _>(self)
    }
}

impl<F, I, St, P, T> Serialize<[F]> for core::iter::Scan<I, St, P>
where
    F: Formula,
    I: Iterator,
    P: FnMut(&mut St, I::Item) -> Option<T>,
    T: Serialize<F>,
{
    #[inline(always)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_iter_to_slice!(F : self => ser)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<usize> {
        default_iter_fast_sizes::<F, _>(self)
    }
}

impl<F, I, T> Serialize<[F]> for core::iter::Skip<I>
where
    F: Formula,
    I: Iterator<Item = T>,
    T: Serialize<F>,
{
    #[inline(always)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_iter_to_slice!(F : self => ser)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<usize> {
        default_iter_fast_sizes::<F, _>(self)
    }
}

impl<F, I, P, T> Serialize<[F]> for core::iter::SkipWhile<I, P>
where
    F: Formula,
    I: Iterator<Item = T>,
    P: FnMut(&T) -> bool,
    T: Serialize<F>,
{
    #[inline(always)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_iter_to_slice!(F : self => ser)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<usize> {
        default_iter_fast_sizes::<F, _>(self)
    }
}

impl<F, I, T> Serialize<[F]> for core::iter::StepBy<I>
where
    F: Formula,
    I: Iterator<Item = T>,
    T: Serialize<F>,
{
    #[inline(always)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_iter_to_slice!(F : self => ser)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<usize> {
        default_iter_fast_sizes::<F, _>(self)
    }
}

impl<F, T, P> Serialize<[F]> for core::iter::Successors<T, P>
where
    F: Formula,
    P: FnMut(&T) -> Option<T>,
    T: Serialize<F>,
{
    #[inline(always)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_iter_to_slice!(F : self => ser)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<usize> {
        default_iter_fast_sizes::<F, _>(self)
    }
}

impl<F, I, T> Serialize<[F]> for core::iter::Take<I>
where
    F: Formula,
    I: Iterator<Item = T>,
    T: Serialize<F>,
{
    #[inline(always)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_iter_to_slice!(F : self => ser)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<usize> {
        default_iter_fast_sizes::<F, _>(self)
    }
}

impl<F, I, P, T> Serialize<[F]> for core::iter::TakeWhile<I, P>
where
    F: Formula,
    I: Iterator<Item = T>,
    P: FnMut(&T) -> bool,
    T: Serialize<F>,
{
    #[inline(always)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_iter_to_slice!(F : self => ser)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<usize> {
        default_iter_fast_sizes::<F, _>(self)
    }
}

impl<FA, FB, A, B> Serialize<[(FA, FB)]> for core::iter::Zip<A, B>
where
    FA: Formula,
    FB: Formula,
    A: Iterator,
    B: Iterator,
    A::Item: Serialize<FA>,
    B::Item: Serialize<FB>,
{
    #[inline(always)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_iter_to_slice!((FA, FB) : self => ser)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<usize> {
        default_iter_fast_sizes::<(FA, FB), _>(self)
    }
}

pub fn deserialize_from_iter<'de, F, A, T>(de: Deserializer<'de>) -> Result<T, DeserializeError>
where
    F: Formula + ?Sized,
    A: Deserialize<'de, F>,
    T: FromIterator<A>,
{
    let mut iter = de.into_iter::<F, A>()?;
    let mut err = None;
    let value = T::from_iter(core::iter::from_fn(|| match iter.next() {
        None => None,
        Some(Ok(elem)) => Some(elem),
        Some(Err(e)) => {
            err = Some(e);
            None
        }
    }));

    match err {
        None => Ok(value),
        Some(e) => Err(e),
    }
}

pub fn deserialize_extend_iter<'de, F, A, T>(
    value: &mut T,
    de: Deserializer<'de>,
) -> Result<(), DeserializeError>
where
    F: Formula + ?Sized,
    A: Deserialize<'de, F>,
    T: Extend<A>,
{
    let mut iter = de.into_iter::<F, A>()?;
    let mut err = None;
    value.extend(core::iter::from_fn(|| match iter.next() {
        None => None,
        Some(Ok(elem)) => Some(elem),
        Some(Err(e)) => {
            err = Some(e);
            None
        }
    }));

    match err {
        None => Ok(()),
        Some(e) => Err(e),
    }
}
