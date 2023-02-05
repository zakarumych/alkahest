use core::{iter::FusedIterator, marker::PhantomData, mem::size_of};

use crate::{
    formula::Formula,
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
/// from raw bytes with specified `F: `[`Formula`].
pub trait Deserialize<'de, F: Formula + ?Sized> {
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
    #[inline(always)]
    #[must_use]
    pub const fn new(stack: usize, input: &'de [u8]) -> Result<Self, Error> {
        if stack > input.len() {
            return Err(Error::OutOfBounds);
        }
        Ok(Self::new_unchecked(stack, input))
    }

    #[inline(always)]
    #[must_use]
    pub const fn new_unchecked(stack: usize, input: &'de [u8]) -> Self {
        Deserializer { input, stack }
    }

    #[inline(always)]
    #[must_use]
    fn sub<F>(&mut self) -> Self
    where
        F: Formula + ?Sized,
    {
        let sub_stack = match F::MAX_SIZE {
            None => self.stack,
            Some(max_size) => self.stack.min(max_size),
        };
        self.stack -= sub_stack;

        Deserializer {
            input: self.input,
            stack: sub_stack,
        }
    }

    #[inline(always)]
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

    #[inline(always)]
    pub fn read_all_bytes(self) -> &'de [u8] {
        let at = self.input.len() - self.stack;
        &self.input[at..]
    }

    #[inline(always)]
    pub fn read_value<F, T>(&mut self) -> Result<T, Error>
    where
        F: Formula + ?Sized,
        T: Deserialize<'de, F>,
    {
        <T as Deserialize<'de, F>>::deserialize(self.sub::<F>())
    }

    #[inline(always)]
    pub fn read_auto<T>(&mut self) -> Result<T, Error>
    where
        T: Formula + Deserialize<'de, T>,
    {
        self.read_value::<T, T>()
    }

    #[inline(always)]
    pub fn read_in_place<F, T>(&mut self, place: &mut T) -> Result<(), Error>
    where
        F: Formula + ?Sized,
        T: Deserialize<'de, F> + ?Sized,
    {
        <T as Deserialize<'de, F>>::deserialize_in_place(place, self.sub::<F>())
    }

    #[inline(always)]
    pub fn read_auto_in_place<T>(&mut self, place: &mut T) -> Result<(), Error>
    where
        T: Formula + Deserialize<'de, T> + ?Sized,
    {
        self.read_in_place::<T, T>(place)
    }

    #[inline(always)]
    pub fn deref(&mut self) -> Result<Deserializer<'de>, Error> {
        let [address, size] = self.read_auto::<[FixedUsize; 2]>()?;

        if usize::from(address) > self.input.len() {
            return Err(Error::WrongAddress);
        }

        Ok(Deserializer {
            input: &self.input[..address.into()],
            stack: size.into(),
        })
    }

    #[inline(always)]
    pub fn into_iter<F, T>(self) -> Result<DeIter<'de, F, T>, Error>
    where
        F: Formula,
        T: Deserialize<'de, F>,
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

    #[inline(always)]
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
    T: Deserialize<'de, F>,
{
    type Item = Result<T, Error>;

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.count, Some(self.count))
    }

    #[inline]
    fn next(&mut self) -> Option<Result<T, Error>> {
        if self.count == 0 {
            return None;
        }

        let size = F::MAX_SIZE.unwrap_or(0);
        let input = self.input;
        self.count -= 1;
        let end = self.input.len() - size;
        self.input = &self.input[..end];

        let result = T::deserialize(Deserializer::new_unchecked(size, input));
        Some(result)
    }

    #[inline]
    fn count(self) -> usize {
        self.count
    }

    #[inline]
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

    #[inline]
    fn fold<B, Fun>(self, init: B, mut f: Fun) -> B
    where
        Fun: FnMut(B, Result<T, Error>) -> B,
    {
        let end = self.input.len();
        let size = F::MAX_SIZE.unwrap_or(0);
        let mut accum = init;
        for elem in 0..self.count {
            let at = end - size * elem;
            let result = T::deserialize(Deserializer::new_unchecked(size, &self.input[..at]));
            accum = f(accum, result);
        }
        accum
    }
}

impl<'de, F, T> DoubleEndedIterator for DeIter<'de, F, T>
where
    F: Formula,
    T: Deserialize<'de, F>,
{
    #[inline]
    fn next_back(&mut self) -> Option<Result<T, Error>> {
        if self.count == 0 {
            return None;
        }
        self.count -= 1;
        let size = F::MAX_SIZE.unwrap_or(0);
        let at = self.input.len() - size * self.count;
        let input = &self.input[at..];

        Some(T::deserialize(Deserializer::new_unchecked(size, input)))
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Result<T, Error>> {
        if n >= self.count {
            self.count = 0;
            return None;
        }
        self.count -= n;
        self.next_back()
    }

    #[inline]
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
            let result = T::deserialize(Deserializer::new_unchecked(size, &self.input[..at]));
            accum = f(accum, result);
        }
        accum
    }
}

impl<'de, F, T> ExactSizeIterator for DeIter<'de, F, T>
where
    F: Formula,
    T: Deserialize<'de, F>,
{
    #[inline]
    fn len(&self) -> usize {
        self.count
    }
}

impl<'de, F, T> FusedIterator for DeIter<'de, F, T>
where
    F: Formula,
    T: Deserialize<'de, F>,
{
}

pub fn value_size(input: &[u8]) -> Result<usize, Error> {
    if input.len() < FIELD_SIZE {
        return Err(Error::OutOfBounds);
    }

    let mut de = Deserializer::new(FIELD_SIZE, &input[..FIELD_SIZE])?;
    de.read_auto::<FixedUsize>().map(usize::from)
}

pub fn deserialize<'de, F, T>(input: &'de [u8]) -> Result<(T, usize), Error>
where
    F: Formula + ?Sized,
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

    if usize::from(address) > input.len() {
        return Err(Error::OutOfBounds);
    }

    let mut de = Deserializer::new(size.into(), &input[..usize::from(address)])?;
    let value = de.read_value::<F, T>()?;

    Ok((value, address.into()))
}

const FIELD_SIZE: usize = size_of::<FixedUsize>();
const HEADER_SIZE: usize = FIELD_SIZE * 2;
