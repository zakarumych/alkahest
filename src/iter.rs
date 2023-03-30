use core::mem::size_of;

use crate::{
    buffer::Buffer,
    deserialize::{Deserialize, DeserializeError, Deserializer},
    formula::Formula,
    serialize::{field_size_hint, write_slice, Serialize, Sizes},
    size::FixedUsize,
};

const ITER_UPPER: usize = 8;

/// Returns the size of the serialized data if it can be determined fast.
#[inline(always)]
pub fn default_iter_fast_sizes<F, I>(iter: &I) -> Option<Sizes>
where
    F: Formula + ?Sized,
    I: Iterator,
    I::Item: Serialize<F>,
{
    match (F::HEAPLESS, F::MAX_STACK_SIZE) {
        (true, Some(0)) => Some(Sizes::with_stack(size_of::<FixedUsize>())),
        (true, Some(max_stack)) => {
            let (lower, upper) = iter.size_hint();
            match upper {
                Some(upper) if upper == lower => {
                    // Expect this to be the truth.
                    // If not, serialization will fail or produce incorrect results.
                    Some(Sizes::with_stack(lower * max_stack))
                }
                _ => None,
            }
        }
        _ => None,
    }
}

/// Returns the size of the serialized data if it can be determined fast.
#[inline(always)]
pub fn ref_iter_fast_sizes<'a, F, I, T: 'a>(iter: I) -> Option<Sizes>
where
    F: Formula + ?Sized,
    I: Iterator<Item = T>,
    T: Serialize<F>,
{
    match (F::HEAPLESS, F::MAX_STACK_SIZE) {
        (true, Some(0)) => Some(Sizes::with_stack(size_of::<FixedUsize>())),
        (true, Some(max_stack)) => {
            let (lower, upper) = iter.size_hint();
            match upper {
                Some(upper) if upper == lower => {
                    // Expect this to be the truth.
                    // If not, serialization will fail or produce incorrect results.
                    Some(Sizes::with_stack(lower * max_stack))
                }
                _ => None,
            }
        }
        _ => {
            let (_lower, upper) = iter.size_hint();
            if upper.map_or(false, |upper| upper <= ITER_UPPER) {
                let mut sizes = Sizes::ZERO;
                for elem in iter {
                    sizes += field_size_hint::<F>(&elem, false)?;
                }
                return Some(sizes);
            }
            None
        }
    }
}

/// Returns the size of the serialized data if it can be determined fast.
#[inline(always)]
pub fn owned_iter_fast_sizes<'a, F, I, T: 'a>(iter: I) -> Option<Sizes>
where
    F: Formula + ?Sized,
    I: Iterator<Item = &'a T>,
    T: Serialize<F>,
{
    match (F::HEAPLESS, F::MAX_STACK_SIZE) {
        (true, Some(0)) => Some(Sizes::with_stack(size_of::<FixedUsize>())),
        (true, Some(max_stack)) => {
            let (lower, upper) = iter.size_hint();
            match upper {
                Some(upper) if upper == lower => {
                    // Expect this to be the truth.
                    // If not, serialization will fail or produce incorrect results.
                    Some(Sizes::with_stack(lower * max_stack))
                }
                _ => None,
            }
        }
        _ => {
            let (_lower, upper) = iter.size_hint();
            if upper.map_or(false, |upper| upper <= ITER_UPPER) {
                let mut sizes = Sizes::ZERO;
                for elem in iter {
                    sizes += field_size_hint::<F>(elem, false)?;
                }
                return Some(sizes);
            }
            None
        }
    }
}

macro_rules! serialize_iter_to_slice {
    ($F:ty : $self:expr => $sizes:ident, $buffer:ident) => {{
        write_slice::<$F, _, _>($self, $sizes, $buffer)
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
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        serialize_iter_to_slice!(F : self.0 => sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
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
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        serialize_iter_to_slice!(F : self => sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        default_iter_fast_sizes::<F, _>(self)
    }
}

impl<F, T> Serialize<[F]> for core::ops::RangeInclusive<T>
where
    F: Formula,
    T: Serialize<F>,
    core::ops::RangeInclusive<T>: Iterator<Item = T>,
{
    #[inline(always)]
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        serialize_iter_to_slice!(F : self => sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        default_iter_fast_sizes::<F, _>(self)
    }
}

impl<F, X, Y, T> Serialize<[F]> for core::iter::Chain<X, Y>
where
    F: Formula,
    X: Iterator<Item = T>,
    Y: Iterator<Item = T>,
    T: Serialize<F>,
{
    #[inline(always)]
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        serialize_iter_to_slice!(F : self => sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
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
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        serialize_iter_to_slice!(F : self => sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
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
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        serialize_iter_to_slice!(F : self => sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        default_iter_fast_sizes::<F, _>(self)
    }
}

impl<F, T> Serialize<[F]> for core::iter::Empty<T>
where
    F: Formula,
    T: Copy + Serialize<F>,
{
    #[inline(always)]
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        serialize_iter_to_slice!(F : self => sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        Some(Sizes::ZERO)
    }
}

// Typically `usize` is not serializable.
// But lib makes exception for `usize`s that are derived from actual sizes.
impl<F, I, T> Serialize<[(FixedUsize, F)]> for core::iter::Enumerate<I>
where
    F: Formula,
    I: Iterator<Item = T>,
    T: Serialize<F>,
{
    #[inline(always)]
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        serialize_iter_to_slice!((FixedUsize, F) : self => sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        default_iter_fast_sizes::<(FixedUsize, F), _>(self)
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
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        serialize_iter_to_slice!(F : self => sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
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
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        serialize_iter_to_slice!(F : self => sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
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
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        serialize_iter_to_slice!(F : self => sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
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
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        serialize_iter_to_slice!(F : self => sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
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
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        serialize_iter_to_slice!(F : self => sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
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
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        serialize_iter_to_slice!(F : self => sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
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
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        serialize_iter_to_slice!(F : self => sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
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
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        serialize_iter_to_slice!(F : self => sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
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
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        serialize_iter_to_slice!(F : self => sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        default_iter_fast_sizes::<F, _>(self)
    }
}

impl<F, T> Serialize<[F]> for core::iter::Once<T>
where
    F: Formula,
    T: Serialize<F>,
{
    #[inline(always)]
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        serialize_iter_to_slice!(F : self => sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
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
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        serialize_iter_to_slice!(F : self => sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
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
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        serialize_iter_to_slice!(F : self => sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
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
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        serialize_iter_to_slice!(F : self => sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
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
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        serialize_iter_to_slice!(F : self => sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
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
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        serialize_iter_to_slice!(F : self => sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
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
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        serialize_iter_to_slice!(F : self => sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
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
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        serialize_iter_to_slice!(F : self => sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
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
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        serialize_iter_to_slice!(F : self => sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
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
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        serialize_iter_to_slice!(F : self => sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
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
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        serialize_iter_to_slice!(F : self => sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        default_iter_fast_sizes::<F, _>(self)
    }
}

impl<FX, FY, X, Y> Serialize<[(FX, FY)]> for core::iter::Zip<X, Y>
where
    FX: Formula,
    FY: Formula,
    X: Iterator,
    Y: Iterator,
    X::Item: Serialize<FX>,
    Y::Item: Serialize<FY>,
{
    #[inline(always)]
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        serialize_iter_to_slice!((FX, FY) : self => sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        default_iter_fast_sizes::<(FX, FY), _>(self)
    }
}

/// Deserialize `FromIterator` value from slice formula.
pub fn deserialize_from_iter<'de, F, A, T>(de: Deserializer<'de>) -> Result<T, DeserializeError>
where
    F: Formula + ?Sized,
    A: Deserialize<'de, F>,
    T: FromIterator<A>,
{
    let mut iter = de.into_unsized_iter::<F, A>();
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

/// Deserialize into `Extend` value from slice formula.
#[inline]
pub fn deserialize_extend_iter<'de, F, A, T>(
    value: &mut T,
    de: Deserializer<'de>,
) -> Result<(), DeserializeError>
where
    F: Formula + ?Sized,
    A: Deserialize<'de, F>,
    T: Extend<A>,
{
    let mut iter = de.into_unsized_iter::<F, A>();
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
