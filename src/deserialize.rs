use core::{any::type_name, iter::FusedIterator, marker::PhantomData, str::Utf8Error};

use crate::{
    formula::{reference_size, unwrap_size, Formula},
    size::{deserialize_usize, FixedIsizeType, FixedUsizeType, SIZE_STACK},
};

#[inline(never)]
#[cold]
pub(crate) const fn cold_err<T>(e: DeserializeError) -> Result<T, DeserializeError> {
    Err(e)
}

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

    /// Signals that deserialization of integer value fails due to
    /// destination type being too small.
    ///
    /// This can happen when deserializing `Vlq` formula
    /// into fixed-size integer type.
    IntegerOverflow,

    /// Data is incompatible with the type to be deserialized.
    Incompatible,
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
    ///
    /// # Errors
    ///
    /// Returns `DeserializeError` if deserialization fails.
    fn deserialize(deserializer: Deserializer<'de>) -> Result<Self, DeserializeError>
    where
        Self: Sized;

    /// Deserializes value in-place provided deserializer.
    /// Overwrites `self` with data from the `input`.
    ///
    /// The value appears at the end of the slice.
    /// And referenced values are addressed from the beginning of the slice.
    ///
    /// # Errors
    ///
    /// Returns `DeserializeError` if deserialization fails.
    fn deserialize_in_place(
        &mut self,
        deserializer: Deserializer<'de>,
    ) -> Result<(), DeserializeError>;
}

/// Deserializer from raw bytes.
/// Provides methods for deserialization of values.
#[must_use = "Deserializer should be used to deserialize values"]
#[derive(Clone)]
pub struct Deserializer<'de> {
    /// Input buffer sub-slice usable for deserialization.
    input: &'de [u8],
    stack: usize,
}

impl<'de> Deserializer<'de> {
    /// Creates new deserializer from input buffer.
    ///
    /// # Errors
    ///
    /// Returns `DeserializeError::OutOfBounds` if
    /// `stack` is greater than `input.len()`.
    #[inline(always)]
    pub const fn new(stack: usize, input: &'de [u8]) -> Result<Self, DeserializeError> {
        if stack > input.len() {
            return cold_err(DeserializeError::OutOfBounds);
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
    pub(crate) fn sub(&mut self, stack: usize) -> Result<Self, DeserializeError> {
        if self.stack < stack {
            return cold_err(DeserializeError::WrongLength);
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
    ///
    /// # Errors
    ///
    /// Returns `DeserializeError` if not enough bytes on stack.
    #[inline(always)]
    pub fn read_bytes(&mut self, len: usize) -> Result<&'de [u8], DeserializeError> {
        if len > self.stack {
            return cold_err(DeserializeError::WrongLength);
        }
        let at = self.input.len() - len;
        let (head, tail) = self.input.split_at(at);
        self.input = head;
        self.stack -= len;
        Ok(tail)
    }

    /// Reads specified number of bytes from the input buffer.
    /// Returns slice of bytes.
    /// Advances the input buffer.
    ///
    /// # Errors
    ///
    /// Returns `DeserializeError` if stack is empty.
    #[inline(always)]
    pub fn read_byte(&mut self) -> Result<u8, DeserializeError> {
        if self.stack == 0 {
            return cold_err(DeserializeError::WrongLength);
        }

        let [head @ .., last] = self.input else {
            unreachable!();
        };
        self.input = head;
        self.stack -= 1;
        Ok(*last)
    }

    /// Reads specified number of bytes from the input buffer.
    /// Returns slice of bytes.
    /// Advances the input buffer.
    ///
    /// # Errors
    ///
    /// Returns `DeserializeError` if not enough bytes on stack.
    #[inline(always)]
    pub fn read_byte_array<const N: usize>(&mut self) -> Result<[u8; N], DeserializeError> {
        if N > self.stack {
            return cold_err(DeserializeError::WrongLength);
        }
        let at = self.input.len() - N;

        let (head, tail) = self.input.split_at(at);
        self.input = head;
        self.stack -= N;

        let mut array = [0; N];
        array.copy_from_slice(tail);
        Ok(array)
    }

    /// Reads the rest of the input buffer as bytes.
    #[must_use]
    #[inline(always)]
    pub fn read_all_bytes(self) -> &'de [u8] {
        let at = self.input.len() - self.stack;
        &self.input[at..]
    }

    /// Reads and deserializes usize from the input buffer.
    /// Advances the input buffer.
    ///
    /// # Errors
    ///
    /// Returns `DeserializeError` if deserialization fails.
    #[inline(always)]
    pub fn read_usize(&mut self) -> Result<usize, DeserializeError> {
        deserialize_usize(self.sub(SIZE_STACK)?)
    }

    /// Reads and deserializes field from the input buffer.
    /// Advances the input buffer.
    ///
    /// # Errors
    ///
    /// Returns `DeserializeError` if deserialization fails.
    #[inline(always)]
    pub fn read_value<F, T>(&mut self, last: bool) -> Result<T, DeserializeError>
    where
        F: Formula + ?Sized,
        T: Deserialize<'de, F>,
    {
        let stack = match (F::MAX_STACK_SIZE, F::EXACT_SIZE, last) {
            (None, _, false) => self.read_usize()?,
            (None, _, true) => self.stack,
            (Some(max_stack), false, true) => max_stack.min(self.stack),
            (Some(max_stack), _, _) => max_stack,
        };

        <T as Deserialize<'de, F>>::deserialize(self.sub(stack)?)
    }

    /// Reads and deserializes field from the back of input buffer.
    /// Advances the input buffer.
    ///
    /// # Errors
    ///
    /// Returns `DeserializeError` if deserialization fails.
    #[inline(always)]
    pub fn read_back_value<F, T>(&mut self) -> Result<T, DeserializeError>
    where
        F: Formula + ?Sized,
        T: Deserialize<'de, F>,
    {
        let stack = unwrap_size(F::MAX_STACK_SIZE);

        if self.stack < stack {
            self.stack = 0;
            return cold_err(DeserializeError::WrongLength);
        }

        let input_back = &self.input[..self.input.len() - self.stack + stack];
        self.stack -= stack;

        let sub = Deserializer::new_unchecked(stack, input_back);
        <T as Deserialize<'de, F>>::deserialize(sub)
    }

    /// Reads and deserializes field from the input buffer in-place.
    ///
    /// # Errors
    ///
    /// Returns `DeserializeError` if deserialization fails.
    #[inline(always)]
    pub fn read_in_place<F, T>(&mut self, place: &mut T, last: bool) -> Result<(), DeserializeError>
    where
        F: Formula + ?Sized,
        T: Deserialize<'de, F> + ?Sized,
    {
        let stack = match (last, F::MAX_STACK_SIZE) {
            (true, _) => self.stack,
            (false, Some(max_stack)) => max_stack,
            (false, None) => self.read_usize()?,
        };

        <T as Deserialize<'de, F>>::deserialize_in_place(place, self.sub(stack)?)
    }

    /// Reads and deserializes reference from the input buffer.
    ///
    /// # Errors
    ///
    /// Returns `DeserializeError` if reference is out of bounds
    /// or has address larger that self.
    #[inline]
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
    ///
    /// # Panics
    ///
    /// Panics if formula is not sized.
    #[inline(always)]
    pub fn into_sized_iter<F, T>(mut self) -> SizedDeIter<'de, F, T>
    where
        F: Formula + ?Sized,
        T: Deserialize<'de, F>,
    {
        let upper = match F::MAX_STACK_SIZE {
            None => panic!("Formula must be sized"),
            Some(0) => self.read_usize().unwrap_or(0),
            Some(max_stack) => self.stack / max_stack,
        };

        assert!(self.stack <= self.input.len());
        DeIter {
            de: self,
            marker: PhantomData,
            upper,
        }
    }

    /// Converts deserializer into iterator over deserialized values with
    /// specified formula.
    #[inline(always)]
    #[allow(clippy::missing_panics_doc)]
    pub fn into_unsized_iter<F, T>(mut self) -> DeIter<'de, F, T>
    where
        F: Formula + ?Sized,
        T: Deserialize<'de, F>,
    {
        let upper = match F::MAX_STACK_SIZE {
            None => self.stack / SIZE_STACK,
            Some(0) => self.read_usize().unwrap_or(0),
            Some(max_stack) => self.stack / max_stack,
        };

        assert!(self.stack <= self.input.len());
        DeIter {
            de: self,
            marker: PhantomData,
            upper,
        }
    }

    /// Converts deserializer into iterator over deserialized values with
    /// specified formula.
    /// The formula must be sized and size must match.
    ///
    /// # Panics
    ///
    /// Panics if formula is not sized.
    #[inline(always)]
    pub fn into_sized_array_iter<F, T>(self, len: usize) -> SizedDeIter<'de, F, T>
    where
        F: Formula + ?Sized,
        T: Deserialize<'de, F>,
    {
        assert!(F::MAX_STACK_SIZE.is_some(), "Formula must be sized");

        assert!(self.stack <= self.input.len());
        DeIter {
            de: self,
            marker: PhantomData,
            upper: len,
        }
    }

    /// Converts deserializer into iterator over deserialized values with
    /// specified formula.
    #[inline(always)]
    #[allow(clippy::missing_panics_doc)]
    pub fn into_unsized_array_iter<F, T>(self, len: usize) -> DeIter<'de, F, T>
    where
        F: Formula + ?Sized,
        T: Deserialize<'de, F>,
    {
        assert!(self.stack <= self.input.len());
        DeIter {
            de: self,
            marker: PhantomData,
            upper: len,
        }
    }

    // /// Finishing check for deserializer.
    // #[inline(always)]
    // pub fn finish(self) -> Result<(), DeserializeError> {
    //     if self.stack == 0 {
    //         Ok(())
    //     } else {
    //         Err(DeserializeError::WrongLength)
    //     }
    // }

    /// Skips specified number of values with specified formula.
    #[inline]
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
                    let skip_bytes = self.read_usize()?;
                    self.read_bytes(skip_bytes)?;
                }
            }
            Some(max_stack) => {
                let skip_bytes = max_stack * n;
                self.read_bytes(skip_bytes)?;
            }
        }
        Ok(())
    }
}

pub struct IterSized;
pub struct IterMaybeUnsized;

pub type SizedDeIter<'de, F, T> = DeIter<'de, F, T, IterSized>;

/// Iterator over deserialized values.
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
    #[must_use]
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.upper == 0 || self.stack_empty()
    }

    /// Returns true if no items remains in the iterator.
    #[inline(always)]
    fn stack_empty(&self) -> bool {
        match F::MAX_STACK_SIZE {
            None => self.de.stack < SIZE_STACK,
            Some(0) => false,
            Some(max_stack) => self.de.stack < max_stack,
        }
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
        match F::MAX_STACK_SIZE {
            None => (usize::from(self.de.stack >= SIZE_STACK), Some(self.upper)),
            Some(_) => (self.upper, Some(self.upper)),
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
            Some(_) => self.upper,
        }
    }

    #[inline]
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
            self.upper -= n;
        }
        self.next()
    }

    #[inline]
    fn fold<B, Fun>(mut self, mut init: B, mut f: Fun) -> B
    where
        Fun: FnMut(B, Result<T, DeserializeError>) -> B,
    {
        match F::MAX_STACK_SIZE {
            None => loop {
                if self.de.stack < SIZE_STACK {
                    break;
                }
                let sub = Deserializer::new_unchecked(SIZE_STACK, self.de.input);
                self.de.input = &self.de.input[..self.de.input.len() - SIZE_STACK];

                let stack = match deserialize_usize(sub) {
                    Ok(stack) => stack,
                    Err(err) => {
                        self.de.stack = 0;
                        return f(init, cold_err(err));
                    }
                };
                let sub = Deserializer::new_unchecked(stack, self.de.input);
                self.de.input = &self.de.input[..self.de.input.len() - stack];
                self.de.stack -= SIZE_STACK * stack;

                let result = <T as Deserialize<'de, F>>::deserialize(sub);
                init = f(init, result);
            },
            Some(0) => {
                let sub = Deserializer::new_unchecked(0, self.de.input);
                for _ in 0..self.upper {
                    let result = <T as Deserialize<'de, F>>::deserialize(sub.clone());
                    init = f(init, result);
                }
            }
            Some(stack) => {
                assert_eq!(self.de.stack / stack, self.upper);
                for _ in 0..self.upper {
                    let sub = Deserializer::new_unchecked(stack, self.de.input);
                    self.de.input = &self.de.input[..self.de.input.len() - stack];

                    let result = <T as Deserialize<'de, F>>::deserialize(sub);
                    init = f(init, result);
                }
            }
        }
        init
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
        if Self::is_empty(self) {
            return None;
        }
        let item = self.de.read_back_value::<F, T>();
        self.upper -= 1;
        Some(item)
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Result<T, DeserializeError>> {
        if n > 0 {
            if n >= self.upper {
                self.upper = 0;
                return None;
            }
            self.de.stack -= n * Self::ELEMENT_SIZE;
            self.upper -= n;
        }
        self.next_back()
    }

    #[inline]
    fn rfold<B, Fun>(self, mut init: B, mut f: Fun) -> B
    where
        Fun: FnMut(B, Result<T, DeserializeError>) -> B,
    {
        match Self::ELEMENT_SIZE {
            0 => {
                let sub = Deserializer::new_unchecked(0, self.de.input);
                for _ in 0..self.upper {
                    let result = <T as Deserialize<'de, F>>::deserialize(sub.clone());
                    init = f(init, result);
                }
            }
            stack => {
                assert_eq!(self.de.stack / stack, self.upper);
                let mut end = self.de.input.len() - stack * self.upper;
                for _ in 0..self.upper {
                    end += stack;
                    let sub = Deserializer::new_unchecked(stack, &self.de.input[..end]);

                    let result = <T as Deserialize<'de, F>>::deserialize(sub);
                    init = f(init, result);
                }
            }
        }
        init
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

/// Deserializes value from the input.
/// The value must occupy the whole input slice.
/// The value must be either sized or heap-less.
/// Returns deserialized value.
///
/// # Errors
///
/// Returns `DeserializeError` if deserialization fails.
#[inline(always)]
pub fn deserialize<'de, F, T>(input: &'de [u8]) -> Result<T, DeserializeError>
where
    F: Formula + ?Sized,
    T: Deserialize<'de, F>,
{
    assert!(
        F::HEAPLESS || F::MAX_STACK_SIZE.is_some(),
        "The value must be either sized or heap-less.
        {} is {} {}",
        type_name::<F>(),
        if F::HEAPLESS {
            "heapless but"
        } else {
            "not heapless and"
        },
        if F::MAX_STACK_SIZE.is_some() {
            "sized"
        } else {
            "not sized"
        }
    );

    let stack = match F::MAX_STACK_SIZE {
        None => input.len(),
        Some(max_stack) => max_stack.min(input.len()),
    };

    let de = Deserializer::new_unchecked(stack, input);
    let value = <T as Deserialize<'de, F>>::deserialize(de)?;

    Ok(value)
}

/// Deserializes value from the input.
/// The value must occupy the whole input slice.
/// Returns deserialized value.
///
/// # Errors
///
/// Returns `DeserializeError` if deserialization fails.
#[inline(always)]
pub fn deserialize_with_size<'de, F, T>(
    input: &'de [u8],
    stack: usize,
) -> Result<T, DeserializeError>
where
    F: Formula + ?Sized,
    T: Deserialize<'de, F>,
{
    let de = Deserializer::new_unchecked(stack, input);
    let value = <T as Deserialize<'de, F>>::deserialize(de)?;

    Ok(value)
}

/// Deserializes value from the input.
/// The value must occupy the whole input slice.
/// The value must be either sized or heap-less.
/// Updates value in-place.
///
/// # Errors
///
/// Returns `DeserializeError` if deserialization fails.
#[inline(always)]
pub fn deserialize_in_place<'de, F, T>(
    place: &mut T,
    input: &'de [u8],
) -> Result<(), DeserializeError>
where
    F: Formula + ?Sized,
    T: Deserialize<'de, F> + ?Sized,
{
    let stack = match F::MAX_STACK_SIZE {
        None => input.len(),
        Some(max_stack) => max_stack.min(input.len()),
    };
    let de = Deserializer::new_unchecked(stack, input);
    <T as Deserialize<'de, F>>::deserialize_in_place(place, de)?;

    Ok(())
}

/// Deserializes value from the input.
/// The value must occupy the whole input slice.
/// Updates value in-place.
///
/// # Errors
///
/// Returns `DeserializeError` if deserialization fails.
#[inline(always)]
pub fn deserialize_in_place_with_size<'de, F, T>(
    place: &mut T,
    input: &'de [u8],
    stack: usize,
) -> Result<(), DeserializeError>
where
    F: Formula + ?Sized,
    T: Deserialize<'de, F> + ?Sized,
{
    let de = Deserializer::new_unchecked(stack, input);
    <T as Deserialize<'de, F>>::deserialize_in_place(place, de)?;

    Ok(())
}

#[inline(always)]
pub fn read_reference<F>(input: &[u8], len: usize) -> (usize, usize)
where
    F: Formula + ?Sized,
{
    let reference_size = reference_size::<F>();
    debug_assert!(reference_size <= input.len());

    if F::EXACT_SIZE {
        let mut de = Deserializer::new(reference_size, &input[..reference_size]).unwrap();
        let Ok(address) = de.read_usize() else {
            unreachable!();
        };
        (address, unwrap_size(F::MAX_STACK_SIZE).min(len))
    } else {
        let mut de = Deserializer::new(reference_size, &input[..reference_size]).unwrap();
        let Ok([size, address]) = de.read_value::<[usize; 2], [usize; 2]>(true) else {
            unreachable!();
        };
        (address, size)
    }
}
