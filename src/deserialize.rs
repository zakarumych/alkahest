use core::{iter::FusedIterator, marker::PhantomData, mem::size_of, str::Utf8Error};

use crate::{
    formula::{reference_size, unwrap_size, Formula},
    size::{FixedIsizeType, FixedUsize, FixedUsizeType},
};

/// Error that can occur during deserialization.
#[derive(Clone, Copy, Debug)]
pub enum DeserializeError {
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
    fn deserialize(deserializer: Deserializer<'de>) -> Result<Self, DeserializeError>
    where
        Self: Sized;

    /// Deserializes value in-place provided deserializer.
    /// Overwrites `self` with data from the `input`.
    ///
    /// The value appears at the end of the slice.
    /// And referenced values are addressed from the beginning of the slice.
    fn deserialize_in_place(
        &mut self,
        deserializer: Deserializer<'de>,
    ) -> Result<(), DeserializeError>;
}

/// Deserializer from raw bytes.
/// Provides methods for deserialization of values.
#[must_use]
#[derive(Clone)]
pub struct Deserializer<'de> {
    /// Input buffer sub-slice usable for deserialization.
    input: &'de [u8],
    stack: usize,
}

impl<'de> Deserializer<'de> {
    /// Creates new deserializer from input buffer.
    #[inline(always)]
    pub const fn new(stack: usize, input: &'de [u8]) -> Result<Self, DeserializeError> {
        if stack > input.len() {
            return Err(DeserializeError::OutOfBounds);
        }
        Ok(Self::new_unchecked(stack, input))
    }

    /// Creates new deserializer from input buffer without bounds checking.
    #[inline(always)]
    pub const fn new_unchecked(stack: usize, input: &'de [u8]) -> Self {
        debug_assert!(stack <= input.len());
        Deserializer { input, stack }
    }

    #[inline(always)]
    #[track_caller]
    pub(crate) fn sub(&mut self, stack: usize) -> Result<Self, DeserializeError> {
        if self.stack < stack {
            return Err(DeserializeError::WrongLength);
        }

        let sub = Deserializer::new_unchecked(stack, self.input);

        self.stack -= stack;
        let end = self.input.len() - stack;
        self.input = &self.input[..end];
        Ok(sub)
    }

    /// Reads specified number of bytes from the input buffer.
    /// Returns slice of bytes.
    /// Advances the input buffer.
    #[inline(always)]
    pub fn read_bytes(&mut self, len: usize) -> Result<&'de [u8], DeserializeError> {
        if len > self.stack {
            return Err(DeserializeError::WrongLength);
        }
        let at = self.input.len() - len;
        let (head, tail) = self.input.split_at(at);
        self.input = head;
        self.stack -= len;
        Ok(tail)
    }

    /// Reads the rest of the input buffer as bytes.
    #[inline(always)]
    pub fn read_all_bytes(self) -> &'de [u8] {
        let at = self.input.len() - self.stack;
        &self.input[at..]
    }

    /// Reads and deserializes field from the input buffer.
    /// Advances the input buffer.
    #[inline(always)]
    pub fn read_value<F, T>(&mut self, last: bool) -> Result<T, DeserializeError>
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

    /// Reads and deserializes field from the input buffer in-place.
    #[inline(always)]
    pub fn read_in_place<F, T>(&mut self, place: &mut T, last: bool) -> Result<(), DeserializeError>
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

    /// Reads and deserializes reference from the input buffer.
    #[inline(always)]
    pub fn deref<F>(self) -> Result<Deserializer<'de>, DeserializeError>
    where
        F: Formula + ?Sized,
    {
        let reference_size = reference_size::<F>();
        if self.stack < reference_size {
            return Err(DeserializeError::OutOfBounds);
        }

        let (head, tail) = self.input.split_at(self.input.len() - reference_size);
        let (address, size) = read_reference::<F>(tail, head.len());

        if address > head.len() {
            return Err(DeserializeError::WrongAddress);
        }

        let input = &head[..address];

        Deserializer::new(size, input)
    }

    /// Converts deserializer into iterator over deserialized values with
    /// specified formula.
    /// The formula must be sized and size must match.
    #[inline(always)]
    pub fn into_sized_iter<F, T>(self) -> SizedDeIter<'de, F, T>
    where
        F: Formula + ?Sized,
        T: Deserialize<'de, F>,
    {
        let upper = match F::MAX_STACK_SIZE {
            None => panic!("Formula must be sized"),
            Some(0) => self
                .clone()
                .read_value::<FixedUsize, usize>(true)
                .unwrap_or(0),
            Some(max_stack) => (self.stack - 1) / max_stack + 1,
        };

        DeIter {
            de: self,
            marker: PhantomData,
            upper,
        }
    }

    /// Converts deserializer into iterator over deserialized values with
    /// specified formula.
    #[inline(always)]
    pub fn into_unsized_iter<F, T>(self) -> DeIter<'de, F, T>
    where
        F: Formula + ?Sized,
        T: Deserialize<'de, F>,
    {
        let upper = match (F::MAX_STACK_SIZE, self.stack) {
            (_, 0) => 0,
            (None, _) => 1 + (self.stack - 1) / size_of::<FixedUsize>(),
            (Some(0), _) => self
                .clone()
                .read_value::<FixedUsize, usize>(true)
                .unwrap_or(0),
            (Some(max_stack), _) => (self.stack - 1) / max_stack + 1,
        };

        DeIter {
            de: self,
            marker: PhantomData,
            upper,
        }
    }

    /// Finishing check for deserializer.
    #[inline(always)]
    pub fn finish(self) -> Result<(), DeserializeError> {
        if self.stack == 0 {
            Ok(())
        } else {
            Err(DeserializeError::WrongLength)
        }
    }

    /// Skips specified number of values with specified formula.
    #[inline(always)]
    fn skip_values<F>(&mut self, n: usize) -> Result<(), DeserializeError>
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
}

pub struct IterSized;
pub struct IterMaybeUnsized;

pub type SizedDeIter<'de, F, T> = DeIter<'de, F, T, IterSized>;

#[must_use]
pub struct DeIter<'de, F: ?Sized, T, M = IterMaybeUnsized> {
    de: Deserializer<'de>,
    upper: usize,
    marker: PhantomData<fn(&F, M) -> T>,
}

impl<'de, F, T, M> DeIter<'de, F, T, M>
where
    F: Formula + ?Sized,
    T: Deserialize<'de, F>,
{
    /// Returns true if no items remains in the iterator.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.upper == 0 || (F::MAX_STACK_SIZE.is_none() && self.de.stack == 0)
    }
}

impl<'de, F, T, M> Clone for DeIter<'de, F, T, M>
where
    F: ?Sized,
{
    #[inline(always)]
    fn clone(&self) -> Self {
        DeIter {
            de: self.de.clone(),
            marker: PhantomData,
            upper: self.upper,
        }
    }

    #[inline(always)]
    fn clone_from(&mut self, source: &Self) {
        self.de = source.de.clone();
    }
}

impl<'de, F, T, M> Iterator for DeIter<'de, F, T, M>
where
    F: Formula + ?Sized,
    T: Deserialize<'de, F>,
{
    type Item = Result<T, DeserializeError>;

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.is_empty() {
            return (0, Some(0));
        }
        match F::MAX_STACK_SIZE {
            None => (0, Some(1 + (self.de.stack - 1) / size_of::<FixedUsize>())),
            Some(0) => (self.upper, Some(self.upper)),
            Some(max_stack) => {
                let count = (self.de.stack - 1) / max_stack + 1;
                (count, Some(count))
            }
        }
    }

    #[inline(always)]
    fn next(&mut self) -> Option<Result<T, DeserializeError>> {
        if self.is_empty() {
            return None;
        }
        let item = self.de.read_value::<F, T>(false);
        self.upper -= 1;
        Some(item)
    }

    #[inline(always)]
    fn count(self) -> usize {
        match F::MAX_STACK_SIZE {
            None => self.fold(0, |acc, _| acc + 1),
            Some(0) => self.upper,
            Some(_) => self.upper,
        }
    }

    #[inline(always)]
    fn nth(&mut self, n: usize) -> Option<Result<T, DeserializeError>> {
        if n > 0 {
            if n >= self.upper {
                self.upper = 0;
                return None;
            }
            if let Err(err) = self.de.skip_values::<F>(n) {
                self.upper = 0;
                return Some(Err(err));
            }
        }
        self.next()
    }

    #[inline(always)]
    fn fold<B, Fun>(mut self, init: B, mut f: Fun) -> B
    where
        Fun: FnMut(B, Result<T, DeserializeError>) -> B,
    {
        let mut accum = init;
        loop {
            if self.is_empty() {
                return accum;
            }
            let result = self.de.read_value::<F, T>(false);
            if let Err(DeserializeError::WrongLength) = result {
                self.upper = 0;
                return accum;
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
    #[inline(always)]
    fn next_back(&mut self) -> Option<Result<T, DeserializeError>> {
        if self.upper == 0 || self.de.stack < Self::ELEMENT_SIZE {
            return None;
        }

        let last = &self.de.input[..self.de.input.len() - self.de.stack + Self::ELEMENT_SIZE];

        Some(
            Deserializer::new(Self::ELEMENT_SIZE, last)
                .unwrap()
                .read_value::<F, T>(false),
        )
    }

    #[inline(always)]
    fn nth_back(&mut self, n: usize) -> Option<Result<T, DeserializeError>> {
        if n > 0 {
            if self.de.stack < (n * Self::ELEMENT_SIZE) {
                self.upper = 0;
                return None;
            }
            self.de.stack -= (n - 1) * Self::ELEMENT_SIZE;
        }
        self.next_back()
    }

    #[inline(always)]
    fn rfold<B, Fun>(self, init: B, mut f: Fun) -> B
    where
        Fun: FnMut(B, Result<T, DeserializeError>) -> B,
    {
        let mut accum = init;
        let mut de = self.de;
        loop {
            if self.upper == 0 || de.stack < Self::ELEMENT_SIZE {
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
    #[inline(always)]
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
#[inline(always)]
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

#[inline(always)]
pub fn deserialize<'de, F, T>(input: &'de [u8]) -> Result<(T, usize), DeserializeError>
where
    F: Formula + ?Sized,
    T: Deserialize<'de, F>,
{
    let reference_size = reference_size::<F>();

    if input.len() < reference_size {
        return Err(DeserializeError::OutOfBounds);
    }

    let (address, size) = read_reference::<F>(input, input.len() - reference_size);

    if size > address {
        return Err(DeserializeError::WrongAddress);
    }

    if address > input.len() {
        return Err(DeserializeError::OutOfBounds);
    }

    let de = Deserializer::new_unchecked(size, &input[..address]);
    let value = <T as Deserialize<'de, F>>::deserialize(de)?;

    Ok((value, address))
}

#[inline(always)]
pub fn deserialize_in_place<'de, F, T>(
    place: &mut T,
    input: &'de [u8],
) -> Result<usize, DeserializeError>
where
    F: Formula + ?Sized,
    T: Deserialize<'de, F> + ?Sized,
{
    let reference_size = reference_size::<F>();

    if input.len() < reference_size {
        return Err(DeserializeError::OutOfBounds);
    }

    let (address, size) = read_reference::<F>(input, input.len() - reference_size);

    if size > address {
        return Err(DeserializeError::WrongAddress);
    }

    if address > input.len() {
        return Err(DeserializeError::OutOfBounds);
    }

    let de = Deserializer::new_unchecked(size, &input[..address]);
    <T as Deserialize<'de, F>>::deserialize_in_place(place, de)?;

    Ok(address)
}

#[inline(always)]
fn read_reference<F>(input: &[u8], len: usize) -> (usize, usize)
where
    F: Formula + ?Sized,
{
    let reference_size = reference_size::<F>();
    debug_assert!(reference_size <= input.len());

    match (F::MAX_STACK_SIZE, F::EXACT_SIZE) {
        (Some(0), _) => {
            // do nothing
            (0, 0)
        }
        (Some(max_stack), true) => {
            let mut de = Deserializer::new(reference_size, &input[..reference_size]).unwrap();
            let Ok(address) = de.read_value::<FixedUsize, usize>(true) else { unreachable!(); };
            (address, max_stack.min(len))
        }
        _ => {
            let mut de = Deserializer::new(reference_size, &input[..reference_size]).unwrap();
            let Ok([size, address]) = de.read_value::<[FixedUsize; 2], [usize; 2]>(true) else { unreachable!(); };
            (address, size)
        }
    }
}
