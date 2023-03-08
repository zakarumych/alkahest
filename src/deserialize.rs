use core::{iter::FusedIterator, marker::PhantomData, mem::size_of, str::Utf8Error};

use crate::{
    cold::{cold, err},
    formula::{unwrap_size, Formula},
    serialize::reference_size,
    size::{FixedIsizeType, FixedUsize, FixedUsizeType},
};

/// DeserializeError that can occur during deserialization.
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

#[must_use]
#[derive(Clone)]
pub struct Deserializer<'de> {
    /// Input buffer sub-slice usable for deserialization.
    input: &'de [u8],
    stack: usize,
}

impl<'de> Deserializer<'de> {
    #[must_use]
    #[inline(always)]
    pub fn new(stack: usize, input: &'de [u8]) -> Result<Self, DeserializeError> {
        if stack > input.len() {
            return err(DeserializeError::OutOfBounds);
        }
        Ok(Self::new_unchecked(stack, input))
    }

    #[must_use]
    #[inline(always)]
    pub const fn new_unchecked(stack: usize, input: &'de [u8]) -> Self {
        debug_assert!(stack <= input.len());
        Deserializer { input, stack }
    }

    #[must_use]
    #[inline(always)]
    #[track_caller]
    pub(crate) fn sub(&mut self, stack: usize) -> Result<Self, DeserializeError> {
        if self.stack < stack {
            return err(DeserializeError::WrongLength);
        }

        let sub = Deserializer::new_unchecked(stack, self.input);

        self.stack -= stack;
        let end = self.input.len() - stack;
        self.input = &self.input[..end];
        Ok(sub)
    }

    #[inline(always)]
    pub fn read_bytes(&mut self, len: usize) -> Result<&'de [u8], DeserializeError> {
        if len > self.stack {
            return err(DeserializeError::WrongLength);
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

    #[inline(always)]
    pub fn skip_values<F>(&mut self, n: usize) -> Result<(), DeserializeError>
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

    #[inline(always)]
    pub fn deref<F>(self) -> Result<Deserializer<'de>, DeserializeError>
    where
        F: Formula + ?Sized,
    {
        let reference_size = reference_size::<F>();
        if self.stack < reference_size {
            return err(DeserializeError::OutOfBounds);
        }
        if self.stack != reference_size {
            return err(DeserializeError::WrongLength);
        }

        let (head, tail) = self.input.split_at(self.input.len() - reference_size);
        let (address, size) = read_reference::<F>(tail, head.len());

        if address > head.len() {
            return err(DeserializeError::WrongAddress);
        }

        let input = &head[..address];

        Deserializer::new(size, input)
    }

    #[inline(always)]
    pub fn into_sized_iter<F, T>(self, max_stack: usize) -> SizedDeIter<'de, F, T>
    where
        F: Formula + ?Sized,
        T: Deserialize<'de, F>,
    {
        debug_assert_eq!(F::MAX_STACK_SIZE, Some(max_stack));

        DeIter {
            de: self,
            marker: PhantomData,
        }
    }

    #[inline(always)]
    pub fn into_unsized_iter<F, T>(self) -> DeIter<'de, F, T>
    where
        F: Formula + ?Sized,
        T: Deserialize<'de, F>,
    {
        DeIter {
            de: self,
            marker: PhantomData,
        }
    }

    #[inline(always)]
    pub fn finish(self) -> Result<(), DeserializeError> {
        if self.stack == 0 {
            Ok(())
        } else {
            err(DeserializeError::WrongLength)
        }
    }
}

pub struct IterSized;
pub struct IterMaybeUnsized;

pub type SizedDeIter<'de, F, T> = DeIter<'de, F, T, IterSized>;

#[must_use]
#[repr(transparent)]
pub struct DeIter<'de, F: ?Sized, T, M = IterMaybeUnsized> {
    de: Deserializer<'de>,
    marker: PhantomData<fn(&F, M) -> T>,
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

    #[inline(always)]
    fn next(&mut self) -> Option<Result<T, DeserializeError>> {
        if self.de.stack == 0 {
            return None;
        }

        Some(self.de.read_value::<F, T>(false))
    }

    #[inline(always)]
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

    #[inline(always)]
    fn nth(&mut self, n: usize) -> Option<Result<T, DeserializeError>> {
        if n > 0 {
            if let Err(_) = self.de.skip_values::<F>(n) {
                return None;
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
            let result = self.de.read_value::<F, T>(false);
            if let Err(DeserializeError::WrongLength) = result {
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
    #[inline(always)]
    fn next_back(&mut self) -> Option<Result<T, DeserializeError>> {
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

    #[inline(always)]
    fn nth_back(&mut self, n: usize) -> Option<Result<T, DeserializeError>> {
        if n > 0 {
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
        return err(DeserializeError::OutOfBounds);
    }

    let (address, size) = read_reference::<F>(input, input.len() - reference_size);

    if size > address {
        return err(DeserializeError::WrongAddress);
    }

    if address > input.len() {
        return err(DeserializeError::OutOfBounds);
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
        return err(DeserializeError::OutOfBounds);
    }

    let (address, size) = read_reference::<F>(input, input.len() - reference_size);

    if size > address {
        return err(DeserializeError::WrongAddress);
    }

    if address > input.len() {
        return err(DeserializeError::OutOfBounds);
    }

    let de = Deserializer::new_unchecked(size.into(), &input[..address]);
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
