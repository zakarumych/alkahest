use crate::{
    formula::Formula,
    serialize::{Serialize, Serializer},
    size::FixedUsize,
    slice::{default_iter_fast_sizes, default_iter_fast_sizes_unchecked},
};

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
        let mut ser = ser.into();
        for elem in self.0 {
            ser.write_value::<F, T>(elem, false)?;
        }
        ser.finish()
    }

    #[inline(always)]
    fn fast_sizes(&self) -> Option<usize> {
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
        let mut ser = ser.into();
        for elem in self {
            ser.write_value::<F, T>(elem, false)?;
        }
        ser.finish()
    }

    #[inline(always)]
    fn fast_sizes(&self) -> Option<usize> {
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
        let mut ser = ser.into();
        for elem in self {
            ser.write_value::<F, T>(elem, false)?;
        }
        ser.finish()
    }

    #[inline(always)]
    fn fast_sizes(&self) -> Option<usize> {
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
    fn serialize<S>(mut self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut ser = ser.into();
        self.try_for_each(|elem| ser.write_value::<F, _>(elem, false))?;
        ser.finish()
    }

    #[inline(always)]
    fn fast_sizes(&self) -> Option<usize> {
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
    fn serialize<S>(mut self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut ser = ser.into();
        self.try_for_each(|elem| ser.write_value::<F, _>(elem, false))?;
        ser.finish()
    }

    #[inline(always)]
    fn fast_sizes(&self) -> Option<usize> {
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
    fn serialize<S>(mut self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut ser = ser.into();
        self.try_for_each(|elem| ser.write_value::<F, _>(elem, false))?;
        ser.finish()
    }

    #[inline(always)]
    fn fast_sizes(&self) -> Option<usize> {
        default_iter_fast_sizes::<F, _>(self)
    }
}

impl<F, T> Serialize<[F]> for core::iter::Empty<T>
where
    F: Formula,
    T: Copy + Serialize<[F]>,
{
    #[inline(always)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        ser.into().finish()
    }

    #[inline(always)]
    fn fast_sizes(&self) -> Option<usize> {
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
    fn serialize<S>(mut self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut ser = ser.into();
        self.try_for_each(|(idx, elem)| {
            ser.write_value::<FixedUsize, _>(FixedUsize::truncate_unchecked(idx), false)?;
            ser.write_value::<F, _>(elem, false)
        })?;
        ser.finish()
    }

    #[inline(always)]
    fn fast_sizes(&self) -> Option<usize> {
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
    fn serialize<S>(mut self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut ser = ser.into();
        self.try_for_each(|elem| ser.write_value::<F, _>(elem, false))?;
        ser.finish()
    }

    #[inline(always)]
    fn fast_sizes(&self) -> Option<usize> {
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
    fn serialize<S>(mut self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut ser = ser.into();
        self.try_for_each(|elem| ser.write_value::<F, _>(elem, false))?;
        ser.finish()
    }

    #[inline(always)]
    fn fast_sizes(&self) -> Option<usize> {
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
    fn serialize<S>(mut self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut ser = ser.into();
        self.try_for_each(|elem| ser.write_value::<F, _>(elem, false))?;
        ser.finish()
    }

    #[inline(always)]
    fn fast_sizes(&self) -> Option<usize> {
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
    fn serialize<S>(mut self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut ser = ser.into();
        self.try_for_each(|elem| ser.write_value::<F, _>(elem, false))?;
        ser.finish()
    }

    #[inline(always)]
    fn fast_sizes(&self) -> Option<usize> {
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
    fn serialize<S>(mut self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut ser = ser.into();
        self.try_for_each(|elem| ser.write_value::<F, _>(elem, false))?;
        ser.finish()
    }

    #[inline(always)]
    fn fast_sizes(&self) -> Option<usize> {
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
    fn serialize<S>(mut self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut ser = ser.into();
        self.try_for_each(|elem| ser.write_value::<F, _>(elem, false))?;
        ser.finish()
    }

    #[inline(always)]
    fn fast_sizes(&self) -> Option<usize> {
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
    fn serialize<S>(mut self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut ser = ser.into();
        self.try_for_each(|elem| ser.write_value::<F, _>(elem, false))?;
        ser.finish()
    }

    #[inline(always)]
    fn fast_sizes(&self) -> Option<usize> {
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
    fn serialize<S>(mut self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut ser = ser.into();
        self.try_for_each(|elem| ser.write_value::<F, _>(elem, false))?;
        ser.finish()
    }

    #[inline(always)]
    fn fast_sizes(&self) -> Option<usize> {
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
    fn serialize<S>(mut self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut ser = ser.into();
        self.try_for_each(|elem| ser.write_value::<F, _>(elem, false))?;
        ser.finish()
    }

    #[inline(always)]
    fn fast_sizes(&self) -> Option<usize> {
        default_iter_fast_sizes::<F, _>(self)
    }
}

impl<F, T> Serialize<[F]> for core::iter::Once<T>
where
    F: Formula,
    T: Serialize<F>,
{
    #[inline(always)]
    fn serialize<S>(mut self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut ser = ser.into();
        self.try_for_each(|elem| ser.write_value::<F, _>(elem, false))?;
        ser.finish()
    }

    #[inline(always)]
    fn fast_sizes(&self) -> Option<usize> {
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
    fn serialize<S>(mut self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut ser = ser.into();
        self.try_for_each(|elem| ser.write_value::<F, _>(elem, false))?;
        ser.finish()
    }

    #[inline(always)]
    fn fast_sizes(&self) -> Option<usize> {
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
    fn serialize<S>(mut self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut ser = ser.into();
        self.try_for_each(|elem| ser.write_value::<F, _>(elem, false))?;
        ser.finish()
    }

    #[inline(always)]
    fn fast_sizes(&self) -> Option<usize> {
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
    fn serialize<S>(mut self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut ser = ser.into();
        self.try_for_each(|elem| ser.write_value::<F, _>(elem, false))?;
        ser.finish()
    }

    #[inline(always)]
    fn fast_sizes(&self) -> Option<usize> {
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
    fn serialize<S>(mut self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut ser = ser.into();
        self.try_for_each(|elem| ser.write_value::<F, _>(elem, false))?;
        ser.finish()
    }

    #[inline(always)]
    fn fast_sizes(&self) -> Option<usize> {
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
    fn serialize<S>(mut self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut ser = ser.into();
        self.try_for_each(|elem| ser.write_value::<F, _>(elem, false))?;
        ser.finish()
    }

    #[inline(always)]
    fn fast_sizes(&self) -> Option<usize> {
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
    fn serialize<S>(mut self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut ser = ser.into();
        self.try_for_each(|elem| ser.write_value::<F, _>(elem, false))?;
        ser.finish()
    }

    #[inline(always)]
    fn fast_sizes(&self) -> Option<usize> {
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
    fn serialize<S>(mut self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut ser = ser.into();
        self.try_for_each(|elem| ser.write_value::<F, _>(elem, false))?;
        ser.finish()
    }

    #[inline(always)]
    fn fast_sizes(&self) -> Option<usize> {
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
    fn serialize<S>(mut self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut ser = ser.into();
        self.try_for_each(|elem| ser.write_value::<F, _>(elem, false))?;
        ser.finish()
    }

    #[inline(always)]
    fn fast_sizes(&self) -> Option<usize> {
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
    fn serialize<S>(mut self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut ser = ser.into();
        self.try_for_each(|elem| ser.write_value::<F, _>(elem, false))?;
        ser.finish()
    }

    #[inline(always)]
    fn fast_sizes(&self) -> Option<usize> {
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
    fn serialize<S>(mut self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut ser = ser.into();
        self.try_for_each(|elem| ser.write_value::<F, _>(elem, false))?;
        ser.finish()
    }

    #[inline(always)]
    fn fast_sizes(&self) -> Option<usize> {
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
    fn serialize<S>(mut self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut ser = ser.into();
        self.try_for_each(|(a, b)| {
            ser.write_value::<FA, _>(a, false)?;
            ser.write_value::<FB, _>(b, false)
        })?;
        ser.finish()
    }

    #[inline(always)]
    fn fast_sizes(&self) -> Option<usize> {
        default_iter_fast_sizes::<(FA, FB), _>(self)
    }
}
