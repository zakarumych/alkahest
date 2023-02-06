use core::{iter::FusedIterator, marker::PhantomData, mem::size_of};

use crate::{
    formula::{Formula, NonRefFormula},
    size::{FixedIsizeType, FixedUsize, FixedUsizeType},
};

/// Error that can occur during deserialization.
#[derive(Debug)]
pub enum Error {
    /// Indicates that input buffer is smaller than
    /// expected value length.
    OutOfBounds,

    /// Relative address is invalid.
    WrongAddress,

    /// Incorrect expected value length.
    WrongLength,

    /// Size value exceeds the maximum `usize` for current architecture.
    InvalidUsize(FixedUsizeType),

    /// Size value exceeds the maximum `isize` for current architecture.
    InvalidIsize(FixedIsizeType),
}

/// Trait for types that can be deserialized
/// from raw bytes with specified `F: `[`NonRefFormula`].
pub trait Deserialize<'de, F: NonRefFormula + ?Sized> {
    /// Deserializes value provided deserializer.
    /// Returns deserialized value and the number of bytes consumed from
    /// the and of input.
    ///
    /// The value appears at the end of the slice.
    /// And referenced values are addressed from the beginning of the slice.
    fn deserialize(deserializer: Deserializer<'de>) -> Result<Self, Error>
    where
        Self: Sized;

    /// Deserializes value in-place provided deserializer.
    /// Overwrites `self` with data from the `input`.
    ///
    /// The value appears at the end of the slice.
    /// And referenced values are addressed from the beginning of the slice.
    fn deserialize_in_place(&mut self, deserializer: Deserializer<'de>) -> Result<(), Error>;
}

#[derive(Clone)]
#[must_use]
pub struct Deserializer<'de> {
    /// Input buffer sub-slice usable for deserialization.
    input: &'de [u8],
    stack: usize,
}

impl<'de> Deserializer<'de> {
    #[cfg_attr(feature = "inline-more", inline(always))]
    #[must_use]
    pub const fn new(stack: usize, input: &'de [u8]) -> Result<Self, Error> {
        if stack > input.len() {
            return Err(Error::OutOfBounds);
        }
        Ok(Self::new_unchecked(stack, input))
    }

    #[cfg_attr(feature = "inline-more", inline(always))]
    #[must_use]
    pub const fn new_unchecked(stack: usize, input: &'de [u8]) -> Self {
        debug_assert!(stack <= input.len());
        Deserializer { input, stack }
    }

    #[cfg_attr(feature = "inline-more", inline(always))]
    #[must_use]
    fn sub<F>(&mut self) -> Self
    where
        F: Formula + ?Sized,
    {
        let sub_stack = match F::MAX_SIZE {
            None => self.stack,
            Some(max_size) => self.stack.min(max_size),
        };

        let sub = Deserializer::new_unchecked(sub_stack, self.input);

        self.stack -= sub_stack;
        let at = self.input.len() - sub_stack;
        self.input = &self.input[..at];
        sub
    }

    #[cfg_attr(feature = "inline-more", inline(always))]
    pub fn read_bytes(&mut self, len: usize) -> Result<&'de [u8], Error> {
        if len > self.stack {
            return Err(Error::OutOfBounds);
        }
        let at = self.input.len() - len;
        let (head, tail) = self.input.split_at(at);
        self.input = head;
        self.stack -= len;
        Ok(tail)
    }

    #[cfg_attr(feature = "inline-more", inline(always))]
    pub fn read_all_bytes(self) -> &'de [u8] {
        let at = self.input.len() - self.stack;
        &self.input[at..]
    }

    #[cfg_attr(feature = "inline-more", inline(always))]
    pub fn read_value<F, T>(&mut self) -> Result<T, Error>
    where
        F: Formula + ?Sized,
        T: Deserialize<'de, F::NonRef>,
    {
        F::deserialize(self.sub::<F>())
    }

    #[cfg_attr(feature = "inline-more", inline(always))]
    pub fn read_auto<T>(&mut self) -> Result<T, Error>
    where
        T: NonRefFormula + Deserialize<'de, T>,
    {
        self.read_value::<T, T>()
    }

    #[cfg_attr(feature = "inline-more", inline(always))]
    pub fn read_in_place<F, T>(&mut self, place: &mut T) -> Result<(), Error>
    where
        F: Formula + ?Sized,
        T: Deserialize<'de, F::NonRef> + ?Sized,
    {
        F::deserialize_in_place(place, self.sub::<F>())
    }

    #[cfg_attr(feature = "inline-more", inline(always))]
    pub fn read_auto_in_place<T>(&mut self, place: &mut T) -> Result<(), Error>
    where
        T: NonRefFormula + Deserialize<'de, T> + ?Sized,
    {
        self.read_in_place::<T, T>(place)
    }

    #[cfg_attr(feature = "inline-more", inline(always))]
    pub fn deref(mut self) -> Result<Deserializer<'de>, Error> {
        let [address, size] = self.read_auto::<[FixedUsize; 2]>()?;

        if usize::from(address) > self.input.len() {
            return Err(Error::WrongAddress);
        }

        let input = &self.input[..address.into()];
        self.finish()?;

        Deserializer::new(size.into(), input)
    }

    #[cfg_attr(feature = "inline-more", inline(always))]
    pub fn into_iter<F, T>(self) -> Result<DeIter<'de, F, T>, Error>
    where
        F: Formula,
        T: Deserialize<'de, F::NonRef>,
    {
        let size = F::MAX_SIZE.expect("Sized formula should have some MAX_SIZE");
        if self.stack % size != 0 {
            return Err(Error::WrongLength);
        }
        let count = self.stack / size;
        Ok(DeIter {
            input: self.input,
            count,
            marker: PhantomData,
        })
    }

    #[cfg_attr(feature = "inline-more", inline(always))]
    pub fn finish(self) -> Result<(), Error> {
        if self.stack == 0 {
            Ok(())
        } else {
            Err(Error::WrongLength)
        }
    }
}

pub struct DeIter<'de, F, T> {
    input: &'de [u8],
    count: usize,
    marker: PhantomData<fn(&F) -> T>,
}

impl<'de, F, T> Iterator for DeIter<'de, F, T>
where
    F: Formula,
    T: Deserialize<'de, F::NonRef>,
{
    type Item = Result<T, Error>;

    #[cfg_attr(feature = "inline-more", inline(always))]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.count, Some(self.count))
    }

    #[cfg_attr(feature = "inline-more", inline(always))]
    fn next(&mut self) -> Option<Result<T, Error>> {
        if self.count == 0 {
            return None;
        }

        let size = F::MAX_SIZE.unwrap_or(0);
        let input = self.input;
        self.count -= 1;
        let end = self.input.len() - size;
        self.input = &self.input[..end];

        let result = F::deserialize(Deserializer::new_unchecked(size, input));
        Some(result)
    }

    #[cfg_attr(feature = "inline-more", inline(always))]
    fn count(self) -> usize {
        self.count
    }

    #[cfg_attr(feature = "inline-more", inline(always))]
    fn nth(&mut self, n: usize) -> Option<Result<T, Error>> {
        if n >= self.count {
            self.count = 0;
            return None;
        }
        self.count -= n;
        let size = F::MAX_SIZE.unwrap_or(0);
        let end = self.input.len() - size * n;
        self.input = &self.input[..end];
        self.next()
    }

    #[cfg_attr(feature = "inline-more", inline(always))]
    fn fold<B, Fun>(self, init: B, mut f: Fun) -> B
    where
        Fun: FnMut(B, Result<T, Error>) -> B,
    {
        let end = self.input.len();
        let size = F::MAX_SIZE.unwrap_or(0);
        let mut accum = init;
        for elem in 0..self.count {
            let at = end - size * elem;
            let result = F::deserialize(Deserializer::new_unchecked(size, &self.input[..at]));
            accum = f(accum, result);
        }
        accum
    }
}

impl<'de, F, T> DoubleEndedIterator for DeIter<'de, F, T>
where
    F: Formula,
    T: Deserialize<'de, F::NonRef>,
{
    #[cfg_attr(feature = "inline-more", inline(always))]
    fn next_back(&mut self) -> Option<Result<T, Error>> {
        if self.count == 0 {
            return None;
        }
        self.count -= 1;
        let size = F::MAX_SIZE.unwrap_or(0);
        let at = self.input.len() - size * self.count;
        let input = &self.input[at..];

        Some(F::deserialize(Deserializer::new_unchecked(size, input)))
    }

    #[cfg_attr(feature = "inline-more", inline(always))]
    fn nth_back(&mut self, n: usize) -> Option<Result<T, Error>> {
        if n >= self.count {
            self.count = 0;
            return None;
        }
        self.count -= n;
        self.next_back()
    }

    #[cfg_attr(feature = "inline-more", inline(always))]
    fn rfold<B, Fun>(self, init: B, mut f: Fun) -> B
    where
        Fun: FnMut(B, Result<T, Error>) -> B,
    {
        if self.count == 0 {
            return init;
        }
        let size = F::MAX_SIZE.unwrap_or(0);
        let start = self.input.len() - size * (self.count - 1);
        let mut accum = init;
        for elem in 0..self.count {
            let at = start + size * elem;
            let result = F::deserialize(Deserializer::new_unchecked(size, &self.input[..at]));
            accum = f(accum, result);
        }
        accum
    }
}

impl<'de, F, T> ExactSizeIterator for DeIter<'de, F, T>
where
    F: Formula,
    T: Deserialize<'de, F::NonRef>,
{
    #[cfg_attr(feature = "inline-more", inline(always))]
    fn len(&self) -> usize {
        self.count
    }
}

impl<'de, F, T> FusedIterator for DeIter<'de, F, T>
where
    F: Formula,
    T: Deserialize<'de, F::NonRef>,
{
}

pub fn value_size(input: &[u8]) -> Result<usize, Error> {
    if input.len() < FIELD_SIZE {
        return Err(Error::OutOfBounds);
    }

    let mut de = Deserializer::new(FIELD_SIZE, &input[..FIELD_SIZE])?;
    de.read_auto::<FixedUsize>().map(usize::from)
}

#[inline]
pub fn deserialize<'de, F, T>(input: &'de [u8]) -> Result<(T, usize), Error>
where
    F: Formula + ?Sized,
    T: Deserialize<'de, F::NonRef>,
{
    if input.len() < HEADER_SIZE {
        return Err(Error::OutOfBounds);
    }

    let mut de = Deserializer::new(HEADER_SIZE, &input[..HEADER_SIZE])?;
    let [address, size] = de.read_auto::<[FixedUsize; 2]>()?;

    if size > address {
        return Err(Error::WrongAddress);
    }

    let end = usize::from(address);

    if end > input.len() {
        return Err(Error::OutOfBounds);
    }

    let mut de = Deserializer::new(size.into(), &input[..end])?;
    let value = de.read_value::<F, T>()?;

    Ok((value, end))
}

#[inline]
pub fn deserialize_in_place<'de, F, T>(place: &mut T, input: &'de [u8]) -> Result<usize, Error>
where
    F: NonRefFormula + ?Sized,
    T: Deserialize<'de, F>,
{
    if input.len() < HEADER_SIZE {
        return Err(Error::OutOfBounds);
    }

    let mut de = Deserializer::new(HEADER_SIZE, &input[..HEADER_SIZE])?;
    let [address, size] = de.read_auto::<[FixedUsize; 2]>()?;

    if size > address {
        return Err(Error::WrongAddress);
    }

    let end = usize::from(address);

    if end > input.len() {
        return Err(Error::OutOfBounds);
    }

    let mut de = Deserializer::new(size.into(), &input[..end])?;
    de.read_in_place::<F, T>(place)?;

    Ok(end)
}

const FIELD_SIZE: usize = size_of::<FixedUsize>();
const HEADER_SIZE: usize = FIELD_SIZE * 2;
