use core::{iter::FusedIterator, marker::PhantomData, mem::size_of, str::Utf8Error};

use crate::{
    cold::{cold, err},
    formula::{reference_size, unwrap_size, Formula},
    size::{FixedIsizeType, FixedUsize, FixedUsizeType},
};

/// Error that can occur during deserialization.
#[derive(Clone, Copy, Debug)]
pub enum Error {
    /// Indicates that input buffer is smaller than
    /// expected value length.
    OutOfBounds,

    /// Relative address is invalid.
    WrongAddress,

    /// Incorrect expected value length.
    WrongLength,

    /// Size value exceeds the maximum `usize` for current platform.
    InvalidUsize(FixedUsizeType),

    /// Size value exceeds the maximum `isize` for current platform.
    InvalidIsize(FixedIsizeType),

    /// Enum variant is invalid.
    WrongVariant(u32),

    /// Bytes slice is not UTF8 where `str` is expected.
    NonUtf8(Utf8Error),
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

#[must_use]
#[derive(Clone)]
pub struct Deserializer<'de> {
    /// Input buffer sub-slice usable for deserialization.
    input: &'de [u8],
    stack: usize,
}

impl<'de> Deserializer<'de> {
    #[must_use]
    #[inline(never)]
    pub fn new(stack: usize, input: &'de [u8]) -> Result<Self, Error> {
        if stack > input.len() {
            return err(Error::OutOfBounds);
        }
        Ok(Self::new_unchecked(stack, input))
    }

    #[must_use]
    #[inline(never)]
    pub const fn new_unchecked(stack: usize, input: &'de [u8]) -> Self {
        debug_assert!(stack <= input.len());
        Deserializer { input, stack }
    }

    #[must_use]
    #[inline(never)]
    #[track_caller]
    pub(crate) fn sub(&mut self, stack: usize) -> Result<Self, Error> {
        if self.stack < stack {
            return err(Error::WrongLength);
        }

        let sub = Deserializer::new_unchecked(stack, self.input);

        self.stack -= stack;
        let end = self.input.len() - stack;
        self.input = &self.input[..end];
        Ok(sub)
    }

    #[inline(never)]
    pub fn read_bytes(&mut self, len: usize) -> Result<&'de [u8], Error> {
        if len > self.stack {
            return err(Error::WrongLength);
        }
        let at = self.input.len() - len;
        let (head, tail) = self.input.split_at(at);
        self.input = head;
        self.stack -= len;
        Ok(tail)
    }

    #[inline(never)]
    pub fn read_all_bytes(self) -> &'de [u8] {
        let at = self.input.len() - self.stack;
        &self.input[at..]
    }

    #[inline(never)]
    #[track_caller]
    pub fn read_value<F, T>(&mut self, last: bool) -> Result<T, Error>
    where
        F: Formula + ?Sized,
        T: Deserialize<'de, F>,
    {
        let stack = match (last, F::MAX_STACK_SIZE) {
            (true, _) => self.stack,
            (false, Some(max_stack)) => max_stack,
            (false, None) => self.read_value::<FixedUsize, usize>(false)?,
        };

        <T as Deserialize<'de, F>>::deserialize(self.sub(stack)?)
    }

    #[inline(never)]
    pub fn skip_values<F>(&mut self, n: usize) -> Result<(), Error>
    where
        F: Formula + ?Sized,
    {
        if n == 0 {
            return Ok(());
        }

        match F::MAX_STACK_SIZE {
            None => {
                for _ in 0..n {
                    let skip_bytes = self.read_value::<FixedUsize, usize>(false)?;
                    self.read_bytes(skip_bytes)?;
                }
            }
            Some(max_stack) => {
                let skip_bytes = max_stack * (n - 1);
                self.read_bytes(skip_bytes)?;
            }
        }
        Ok(())
    }

    #[inline(never)]
    pub fn read_in_place<F, T>(&mut self, place: &mut T, last: bool) -> Result<(), Error>
    where
        F: Formula + ?Sized,
        T: Deserialize<'de, F> + ?Sized,
    {
        let stack = match (last, F::MAX_STACK_SIZE) {
            (true, _) => self.stack,
            (false, Some(max_stack)) => max_stack,
            (false, None) => self.read_value::<FixedUsize, usize>(false)?,
        };

        <T as Deserialize<'de, F>>::deserialize_in_place(place, self.sub(stack)?)
    }

    #[inline(never)]
    pub fn deref<F>(self) -> Result<Deserializer<'de>, Error>
    where
        F: Formula + ?Sized,
    {
        let reference_size = reference_size::<F>();
        if self.stack < reference_size {
            return err(Error::OutOfBounds);
        }
        if self.stack != reference_size {
            return err(Error::WrongLength);
        }

        let (head, tail) = self.input.split_at(self.input.len() - reference_size);
        let (address, size) = read_reference::<F>(tail, head.len());

        if address > head.len() {
            return err(Error::WrongAddress);
        }

        let input = &head[..address];

        Deserializer::new(size, input)
    }

    #[inline(never)]
    pub fn into_iter<F, T>(self) -> Result<DeIter<'de, F, T>, Error>
    where
        F: Formula + ?Sized,
        T: Deserialize<'de, F>,
    {
        debug_assert!(F::MAX_STACK_SIZE.is_some());

        Ok(DeIter {
            de: self,
            marker: PhantomData,
        })
    }

    #[inline(never)]
    pub fn into_unsized_iter<F, T>(self) -> Result<UnsizedDeIter<'de, F, T>, Error>
    where
        F: Formula + ?Sized,
        T: Deserialize<'de, F>,
    {
        Ok(DeIter {
            de: self,
            marker: PhantomData,
        })
    }

    #[inline(never)]
    pub fn finish(self) -> Result<(), Error> {
        if self.stack == 0 {
            Ok(())
        } else {
            err(Error::WrongLength)
        }
    }
}

pub struct IterSized;
pub struct IterUnsized;

pub type UnsizedDeIter<'de, F, T> = DeIter<'de, F, T, IterUnsized>;

pub struct DeIter<'de, F: ?Sized, T, M = IterSized> {
    de: Deserializer<'de>,
    marker: PhantomData<fn(&F, M) -> T>,
}

impl<'de, F, T, M> Clone for DeIter<'de, F, T, M>
where
    F: ?Sized,
{
    #[inline(never)]
    fn clone(&self) -> Self {
        DeIter {
            de: self.de.clone(),
            marker: PhantomData,
        }
    }

    #[inline(never)]
    fn clone_from(&mut self, source: &Self) {
        self.de = source.de.clone();
    }
}

impl<'de, F, T, M> Iterator for DeIter<'de, F, T, M>
where
    F: Formula + ?Sized,
    T: Deserialize<'de, F>,
{
    type Item = Result<T, Error>;

    #[inline(never)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.de.stack == 0 {
            return (0, Some(0));
        }
        match F::MAX_STACK_SIZE {
            None => (0, Some((self.de.stack - 1) / size_of::<FixedUsize>())),
            Some(0) => {
                let len = self
                    .de
                    .clone()
                    .read_value::<FixedUsize, usize>(true)
                    .unwrap_or(0);
                (len, Some(len))
            }
            Some(max_stack) => {
                let count = (self.de.stack - 1) / max_stack + 1;
                (count, Some(count))
            }
        }
    }

    #[inline(never)]
    fn next(&mut self) -> Option<Result<T, Error>> {
        if self.de.stack == 0 {
            return None;
        }

        Some(self.de.read_value::<F, T>(false))
    }

    #[inline(never)]
    fn count(self) -> usize {
        match F::MAX_STACK_SIZE {
            None => self.fold(0, |acc, _| acc + 1),
            Some(0) => self
                .de
                .clone()
                .read_value::<FixedUsize, usize>(false)
                .unwrap_or(0),
            Some(max_stack) => (self.de.stack + max_stack - 1) / max_stack,
        }
    }

    #[inline(never)]
    fn nth(&mut self, n: usize) -> Option<Result<T, Error>> {
        if n > 0 {
            if let Err(_) = self.de.skip_values::<F>(n) {
                return None;
            }
        }
        self.next()
    }

    #[inline(never)]
    fn fold<B, Fun>(mut self, init: B, mut f: Fun) -> B
    where
        Fun: FnMut(B, Result<T, Error>) -> B,
    {
        let mut accum = init;
        loop {
            let result = self.de.read_value::<F, T>(false);
            if let Err(Error::WrongLength) = result {
                self.de.input = &[];
                self.de.stack = 0;
                if self.de.stack == 0 {
                    return accum;
                }
                cold();
                return f(accum, result);
            }
            accum = f(accum, result);
        }
    }
}

impl<'de, F, T> DeIter<'de, F, T, IterSized>
where
    F: Formula + ?Sized,
    T: Deserialize<'de, F>,
{
    const ELEMENT_SIZE: usize = unwrap_size(F::MAX_STACK_SIZE);
}

impl<'de, F, T> DoubleEndedIterator for DeIter<'de, F, T, IterSized>
where
    F: Formula + ?Sized,
    T: Deserialize<'de, F>,
{
    #[inline(never)]
    fn next_back(&mut self) -> Option<Result<T, Error>> {
        if self.de.stack < Self::ELEMENT_SIZE {
            return None;
        }

        let last = &self.de.input[..self.de.input.len() - self.de.stack + Self::ELEMENT_SIZE];

        Some(
            Deserializer::new(Self::ELEMENT_SIZE, last)
                .unwrap()
                .read_value::<F, T>(false),
        )
    }

    #[inline(never)]
    fn nth_back(&mut self, n: usize) -> Option<Result<T, Error>> {
        if n > 0 {
            self.de.stack -= (n - 1) * Self::ELEMENT_SIZE;
        }
        self.next_back()
    }

    #[inline(never)]
    fn rfold<B, Fun>(self, init: B, mut f: Fun) -> B
    where
        Fun: FnMut(B, Result<T, Error>) -> B,
    {
        let mut accum = init;
        let mut de = self.de;
        loop {
            if de.stack < Self::ELEMENT_SIZE {
                return accum;
            }

            let last = &de.input[..de.input.len() - de.stack + Self::ELEMENT_SIZE];

            let result = Deserializer::new(Self::ELEMENT_SIZE, last)
                .unwrap()
                .read_value::<F, T>(false);

            accum = f(accum, result);
            de.stack -= Self::ELEMENT_SIZE;
        }
    }
}

impl<'de, F, T> ExactSizeIterator for DeIter<'de, F, T, IterSized>
where
    F: Formula + ?Sized,
    T: Deserialize<'de, F>,
{
    #[inline(never)]
    fn len(&self) -> usize {
        self.size_hint().0
    }
}

impl<'de, F, T, M> FusedIterator for DeIter<'de, F, T, M>
where
    F: Formula + ?Sized,
    T: Deserialize<'de, F>,
{
}

/// Reads size of the value from the input.
/// Returns `None` if the input is too short to determine the size.
#[inline(never)]
pub fn value_size<F>(input: &[u8]) -> Option<usize>
where
    F: Formula + ?Sized,
{
    match F::MAX_STACK_SIZE {
        Some(0) => Some(0),
        _ => {
            if input.len() < size_of::<FixedUsize>() {
                None
            } else {
                let mut de = Deserializer::new_unchecked(
                    size_of::<FixedUsize>(),
                    &input[..size_of::<FixedUsize>()],
                );
                let address = de.read_value::<FixedUsize, usize>(false).unwrap();
                Some(address)
            }
        }
    }
}

#[inline(never)]
pub fn deserialize<'de, F, T>(input: &'de [u8]) -> Result<(T, usize), Error>
where
    F: Formula + ?Sized,
    T: Deserialize<'de, F>,
{
    let reference_size = reference_size::<F>();

    if input.len() < reference_size {
        return err(Error::OutOfBounds);
    }

    let (address, size) = read_reference::<F>(input, input.len() - reference_size);

    if size > address {
        return err(Error::WrongAddress);
    }

    if address > input.len() {
        return err(Error::OutOfBounds);
    }

    let de = Deserializer::new_unchecked(size, &input[..address]);
    let value = <T as Deserialize<'de, F>>::deserialize(de)?;

    Ok((value, address))
}

#[inline(never)]
pub fn deserialize_in_place<'de, F, T>(place: &mut T, input: &'de [u8]) -> Result<usize, Error>
where
    F: Formula + ?Sized,
    T: Deserialize<'de, F> + ?Sized,
{
    let reference_size = reference_size::<F>();

    if input.len() < reference_size {
        return err(Error::OutOfBounds);
    }

    let (address, size) = read_reference::<F>(input, input.len() - reference_size);

    if size > address {
        return err(Error::WrongAddress);
    }

    if address > input.len() {
        return err(Error::OutOfBounds);
    }

    let de = Deserializer::new_unchecked(size.into(), &input[..address]);
    <T as Deserialize<'de, F>>::deserialize_in_place(place, de)?;

    Ok(address)
}

#[inline(never)]
fn read_reference<F>(input: &[u8], len: usize) -> (usize, usize)
where
    F: Formula + ?Sized,
{
    let reference_size = reference_size::<F>();
    debug_assert!(reference_size <= input.len());

    match F::MAX_STACK_SIZE {
        Some(0) => {
            // do nothing
            (0, 0)
        }
        Some(max_stack) => {
            let mut de = Deserializer::new(reference_size, &input[..reference_size]).unwrap();
            let Ok(address) = de.read_value::<FixedUsize, usize>(true) else { unreachable!(); };
            (address, max_stack.min(len))
        }
        None => {
            let mut de = Deserializer::new(reference_size, &input[..reference_size]).unwrap();
            let Ok([size, address]) = de.read_value::<[FixedUsize; 2], [usize; 2]>(true) else { unreachable!(); };
            (address, size)
        }
    }
}
