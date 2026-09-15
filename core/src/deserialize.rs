use core::{fmt, iter::FusedIterator, marker::PhantomData, str::Utf8Error};

use crate::{
    Element,
    element::stack_size,
    formula::{Formula, SizeBound},
};

#[inline]
#[cold]
pub(crate) fn cold_err<T>(e: DeserializeError) -> Result<T, DeserializeError> {
    Err(e)
}

/// Error that can occur during deserialization.
#[derive(Clone, Copy, Debug)]
pub enum DeserializeError {
    /// Indicates that input buffer is smaller than
    /// expected value length.
    OutOfBounds(usize),

    /// Relative address is invalid.
    WrongAddress,

    /// Incorrect expected value length.
    WrongLength,

    /// Size value exceeds the maximum `usize` for current platform.
    TooLarge(u128),

    /// Enum variant is invalid.
    WrongVariant(usize),

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

impl fmt::Display for DeserializeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

pub trait Deserializer<'de> {
    fn read_bytes(&mut self, len: usize) -> Result<&'de [u8], DeserializeError>;

    fn read_byte(&mut self) -> Result<u8, DeserializeError>;

    fn read_byte_array<const N: usize>(&mut self) -> Result<&'de [u8; N], DeserializeError>;

    fn read_usize(&mut self) -> Result<usize, DeserializeError>;

    fn read_value<F, T>(&mut self) -> Result<T, DeserializeError>
    where
        F: Formula + ?Sized,
        T: Deserialize<'de, F>;

    /// Reads and deserializes field from the input buffer in-place.
    ///
    /// # Errors
    ///
    /// Returns `DeserializeError` if deserialization fails.
    fn read_value_in_place<F, T>(&mut self, place: &mut T) -> Result<(), DeserializeError>
    where
        F: Formula + ?Sized,
        T: Deserialize<'de, F> + ?Sized;

    /// Converts deserializer into iterator over deserialized values with specified formula.
    fn into_iter<E, T>(
        self,
        len: usize,
    ) -> impl DoubleEndedIterator<Item = Result<T, DeserializeError>>
    where
        E: Element + ?Sized,
        T: Deserialize<'de, E::Formula>,
        Self: Sized;

    #[doc(hidden)]
    fn input(&self) -> &'de [u8];

    #[doc(hidden)]
    fn at(&self, address: usize) -> Result<impl Deserializer<'de>, DeserializeError>;

    #[doc(hidden)]
    fn size_bytes(&self) -> usize;
}

/// Trait for types that can be deserialized
/// from raw bytes with specified `F: `[`Formula`].
pub trait Deserialize<'de, F: ?Sized> {
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
    fn deserialize<D>(deserializer: D) -> Result<Self, DeserializeError>
    where
        D: Deserializer<'de>,
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
    fn deserialize_in_place<D>(&mut self, deserializer: D) -> Result<(), DeserializeError>
    where
        D: Deserializer<'de>;
}

/// Deserializer from raw bytes.
/// Provides methods for deserialization of values.
#[must_use = "Deserializer should be used to deserialize values"]
#[derive(Clone)]
pub(crate) struct DeserializerImpl<'de, const SIZE_BYTES: usize> {
    /// Input buffer sub-slice usable for deserialization.
    input: &'de [u8],

    #[cfg(debug_assertions)]
    debug_exhausted: bool,
}

impl<'de, const SIZE_BYTES: usize> DeserializerImpl<'de, SIZE_BYTES> {
    /// Creates new deserializer from input buffer.
    #[inline]
    pub const fn new(input: &'de [u8]) -> Self {
        DeserializerImpl {
            input,
            #[cfg(debug_assertions)]
            debug_exhausted: false,
        }
    }

    #[cfg(debug_assertions)]
    #[inline]
    fn debug_validate(&self) {
        assert!(!self.debug_exhausted, "Deserializer used after exhaustion");
    }

    #[inline]
    fn skip_padding<F>(&mut self, new_len: &mut usize)
    where
        F: Formula + ?Sized,
    {
        match const { stack_size::<F, SIZE_BYTES>() } {
            SizeBound::Bounded(max_stack) => {
                debug_assert!(*new_len >= self.input.len().saturating_sub(max_stack));

                #[cfg(debug_assertions)]
                if self.input.len() < max_stack {
                    self.debug_exhausted = true;
                }

                *new_len = self.input.len().saturating_sub(max_stack);
            }
            SizeBound::Exact(exact_stack) => {
                debug_assert_eq!(self.input.len() - exact_stack, *new_len);
            }
            _ => {}
        }
    }
}

impl<'de, const SIZE_BYTES: usize> Deserializer<'de> for DeserializerImpl<'de, SIZE_BYTES> {
    /// Reads specified number of bytes from the input buffer.
    /// Returns slice of bytes.
    /// Advances the input buffer.
    ///
    /// # Errors
    ///
    /// Returns `DeserializeError` if not enough bytes on stack.
    #[inline]
    fn read_bytes(&mut self, len: usize) -> Result<&'de [u8], DeserializeError> {
        #[cfg(debug_assertions)]
        self.debug_validate();

        if len > self.input.len() {
            return cold_err(DeserializeError::WrongLength);
        }
        let at = self.input.len() - len;
        let (head, tail) = self.input.split_at(at);
        self.input = head;
        Ok(tail)
    }

    /// Reads specified number of bytes from the input buffer.
    /// Returns slice of bytes.
    /// Advances the input buffer.
    ///
    /// # Errors
    ///
    /// Returns `DeserializeError` if stack is empty.
    #[inline]
    fn read_byte(&mut self) -> Result<u8, DeserializeError> {
        #[cfg(debug_assertions)]
        self.debug_validate();

        if self.input.is_empty() {
            return cold_err(DeserializeError::WrongLength);
        }

        let [head @ .., last] = self.input else {
            unreachable!();
        };
        self.input = head;
        Ok(*last)
    }

    /// Reads specified number of bytes from the input buffer.
    /// Returns slice of bytes.
    /// Advances the input buffer.
    ///
    /// # Errors
    ///
    /// Returns `DeserializeError` if not enough bytes on stack.
    #[inline]
    fn read_byte_array<const N: usize>(&mut self) -> Result<&'de [u8; N], DeserializeError> {
        #[cfg(debug_assertions)]
        self.debug_validate();

        if N > self.input.len() {
            return cold_err(DeserializeError::WrongLength);
        }

        let at = self.input.len() - N;

        let (head, tail) = self.input.split_at(at);
        self.input = head;

        Ok(tail.as_array().unwrap())
    }

    /// Reads and deserializes usize from the input buffer.
    /// Advances the input buffer.
    ///
    /// # Errors
    ///
    /// Returns `DeserializeError` if deserialization fails.
    #[inline]
    fn read_usize(&mut self) -> Result<usize, DeserializeError> {
        #[cfg(debug_assertions)]
        self.debug_validate();

        let input = simple_try!(self.read_byte_array::<SIZE_BYTES>());
        read_usize::<SIZE_BYTES>(input)
    }

    /// Reads and deserializes field from the input buffer.
    /// Advances the input buffer.
    ///
    /// # Errors
    ///
    /// Returns `DeserializeError` if deserialization fails.
    #[inline]
    fn read_value<F, T>(&mut self) -> Result<T, DeserializeError>
    where
        F: Formula + ?Sized,
        T: Deserialize<'de, F>,
    {
        #[cfg(debug_assertions)]
        self.debug_validate();

        let mut new_len: usize = 0;
        let value = simple_try!(<T as Deserialize<'de, F>>::deserialize(
            TrackingDeserializerImpl::<SIZE_BYTES>::new(self.input, &mut new_len),
        ));

        self.skip_padding::<F>(&mut new_len);

        self.input = &self.input[..new_len];
        Ok(value)
    }

    /// Reads and deserializes field from the input buffer in-place.
    ///
    /// # Errors
    ///
    /// Returns `DeserializeError` if deserialization fails.
    #[inline]
    fn read_value_in_place<F, T>(&mut self, place: &mut T) -> Result<(), DeserializeError>
    where
        F: Formula + ?Sized,
        T: Deserialize<'de, F> + ?Sized,
    {
        #[cfg(debug_assertions)]
        self.debug_validate();

        let mut new_len: usize = 0;
        simple_try!(<T as Deserialize<'de, F>>::deserialize_in_place(
            place,
            TrackingDeserializerImpl::<SIZE_BYTES>::new(self.input, &mut new_len),
        ));

        self.skip_padding::<F>(&mut new_len);

        self.input = &self.input[..new_len];
        Ok(())
    }

    #[allow(refining_impl_trait)]
    fn at(&self, address: usize) -> Result<impl Deserializer<'de>, DeserializeError> {
        if self.input.len() < address {
            cold_err(DeserializeError::WrongAddress)
        } else {
            Ok(DeserializerImpl::<SIZE_BYTES>::new(&self.input[..address]))
        }
    }

    #[allow(refining_impl_trait)]
    #[inline]
    fn input(&self) -> &'de [u8] {
        self.input
    }

    #[doc(hidden)]
    #[inline]
    fn size_bytes(&self) -> usize {
        SIZE_BYTES
    }

    /// Converts deserializer into iterator over deserialized values with specified formula.
    ///
    /// # Panics
    ///
    /// `SIZE_BYTES` must match `self.size_bytes()`
    #[allow(refining_impl_trait)]
    fn into_iter<E, T>(self, len: usize) -> DeIter<'de, E, T, SIZE_BYTES>
    where
        E: Element + ?Sized,
        T: Deserialize<'de, E::Formula>,
        Self: Sized,
    {
        DeIter {
            de: self,
            len,
            marker: PhantomData,
        }
    }
}

/// Deserializer from raw bytes.
/// Provides methods for deserialization of values.
#[must_use = "Deserializer should be used to deserialize values"]
pub struct TrackingDeserializerImpl<'de, 'consumed, const SIZE_BYTES: usize> {
    /// Input buffer sub-slice usable for deserialization.
    inner: DeserializerImpl<'de, SIZE_BYTES>,

    rest: &'consumed mut usize,
}

impl<'de, 'consumed, const SIZE_BYTES: usize> Drop
    for TrackingDeserializerImpl<'de, 'consumed, SIZE_BYTES>
{
    fn drop(&mut self) {
        *self.rest = self.inner.input.len();
    }
}

impl<'de, 'consumed, const SIZE_BYTES: usize> TrackingDeserializerImpl<'de, 'consumed, SIZE_BYTES> {
    #[inline]
    pub const fn new(input: &'de [u8], rest: &'consumed mut usize) -> Self {
        TrackingDeserializerImpl {
            inner: DeserializerImpl::new(input),
            rest,
        }
    }
}

impl<'de, const SIZE_BYTES: usize> Deserializer<'de>
    for TrackingDeserializerImpl<'de, '_, SIZE_BYTES>
{
    /// Reads specified number of bytes from the input buffer.
    /// Returns slice of bytes.
    /// Advances the input buffer.
    ///
    /// # Errors
    ///
    /// Returns `DeserializeError` if not enough bytes on stack.
    #[inline]
    fn read_bytes(&mut self, len: usize) -> Result<&'de [u8], DeserializeError> {
        self.inner.read_bytes(len)
    }

    /// Reads specified number of bytes from the input buffer.
    /// Returns slice of bytes.
    /// Advances the input buffer.
    ///
    /// # Errors
    ///
    /// Returns `DeserializeError` if stack is empty.
    #[inline]
    fn read_byte(&mut self) -> Result<u8, DeserializeError> {
        self.inner.read_byte()
    }

    /// Reads specified number of bytes from the input buffer.
    /// Returns slice of bytes.
    /// Advances the input buffer.
    ///
    /// # Errors
    ///
    /// Returns `DeserializeError` if not enough bytes on stack.
    #[inline]
    fn read_byte_array<const N: usize>(&mut self) -> Result<&'de [u8; N], DeserializeError> {
        self.inner.read_byte_array::<N>()
    }

    /// Reads and deserializes usize from the input buffer.
    /// Advances the input buffer.
    ///
    /// # Errors
    ///
    /// Returns `DeserializeError` if deserialization fails.
    #[inline]
    fn read_usize(&mut self) -> Result<usize, DeserializeError> {
        self.inner.read_usize()
    }

    /// Reads and deserializes field from the input buffer.
    /// Advances the input buffer.
    ///
    /// # Errors
    ///
    /// Returns `DeserializeError` if deserialization fails.
    #[inline]
    fn read_value<F, T>(&mut self) -> Result<T, DeserializeError>
    where
        F: Formula + ?Sized,
        T: Deserialize<'de, F>,
    {
        self.inner.read_value()
    }

    /// Reads and deserializes field from the input buffer in-place.
    ///
    /// # Errors
    ///
    /// Returns `DeserializeError` if deserialization fails.
    #[inline]
    fn read_value_in_place<F, T>(&mut self, place: &mut T) -> Result<(), DeserializeError>
    where
        F: Formula + ?Sized,
        T: Deserialize<'de, F> + ?Sized,
    {
        self.inner.read_value_in_place(place)
    }

    #[inline]
    fn at(&self, address: usize) -> Result<impl Deserializer<'de>, DeserializeError> {
        self.inner.at(address)
    }

    #[allow(refining_impl_trait)]
    #[inline]
    fn input(&self) -> &'de [u8] {
        self.inner.input
    }

    #[doc(hidden)]
    #[inline]
    fn size_bytes(&self) -> usize {
        SIZE_BYTES
    }

    #[allow(refining_impl_trait)]
    fn into_iter<E, T>(self, len: usize) -> DeIter<'de, E, T, SIZE_BYTES>
    where
        E: Element + ?Sized,
        T: Deserialize<'de, E::Formula>,
        Self: Sized,
    {
        DeIter {
            de: self.inner.clone(),
            len,
            marker: PhantomData,
        }
    }
}

/// Iterator over deserialized values.
#[must_use]
pub struct DeIter<'de, E: ?Sized, T, const SIZE_BYTES: usize> {
    de: DeserializerImpl<'de, SIZE_BYTES>,
    len: usize,
    marker: PhantomData<fn(E) -> T>,
}

impl<'de, E, T, const SIZE_BYTES: usize> DeIter<'de, E, T, SIZE_BYTES>
where
    E: Element + ?Sized,
    T: Deserialize<'de, E::Formula>,
{
    /// Returns true if no items remains in the iterator.
    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn read_at(&self, index: usize) -> Result<T, DeserializeError> {
        const {
            assert!(matches!(
                stack_size::<E, SIZE_BYTES>(),
                SizeBound::Bounded(_) | SizeBound::Exact(_)
            ));
        }

        let mut de = self.de.clone();

        match stack_size::<E, SIZE_BYTES>() {
            SizeBound::Bounded(size) | SizeBound::Exact(size) => {
                if size * index <= de.input.len() {
                    let end = de.input.len() - size * index;
                    de.input = &de.input[..end];
                } else {
                    de.input = &[];
                }
                E::deserialize::<T, _>(&mut de)
            }
            SizeBound::Unbounded => {
                let mut de = self.de.clone();
                for _ in 0..index {
                    let _ = simple_try!(E::deserialize::<T, _>(&mut de));
                }
                E::deserialize::<T, _>(&mut de)
            }
        }
    }
}

impl<'de, F, T, const SIZE_BYTES: usize> Clone for DeIter<'de, F, T, SIZE_BYTES>
where
    F: ?Sized,
{
    #[inline]
    fn clone(&self) -> Self {
        DeIter {
            de: self.de.clone(),
            marker: PhantomData,
            len: self.len,
        }
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        self.de = source.de.clone();
        self.len = source.len;
    }
}

impl<'de, E, T, const SIZE_BYTES: usize> Iterator for DeIter<'de, E, T, SIZE_BYTES>
where
    E: Element + ?Sized,
    T: Deserialize<'de, E::Formula>,
{
    type Item = Result<T, DeserializeError>;

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }

    #[inline]
    fn next(&mut self) -> Option<Result<T, DeserializeError>> {
        if self.is_empty() {
            return None;
        }

        match E::deserialize(&mut self.de) {
            Ok(item) => {
                self.len -= 1;
                Some(Ok(item))
            }
            Err(err) => {
                self.len = 0;
                Some(Err(err))
            }
        }
    }

    #[inline]
    fn count(self) -> usize {
        self.len
    }

    #[inline]
    fn last(self) -> Option<Result<T, DeserializeError>> {
        if self.len == 0 {
            return None;
        }
        match self.read_at(self.len - 1) {
            Ok(item) => Some(Ok(item)),
            Err(err) => Some(Err(err)),
        }
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Result<T, DeserializeError>> {
        if n > 0 {
            if n >= self.len {
                self.len = 0;
                return None;
            }

            match stack_size::<E, SIZE_BYTES>() {
                SizeBound::Unbounded => {
                    for _ in 0..n {
                        match self.next() {
                            None => return None,
                            Some(Err(err)) => {
                                self.len = 0;
                                return Some(Err(err));
                            }
                            Some(Ok(_)) => {}
                        }
                    }
                }

                SizeBound::Bounded(size) | SizeBound::Exact(size) => {
                    if size * n <= self.de.input.len() {
                        let end = self.de.input.len() - size * n;
                        self.de.input = &self.de.input[..end]
                    } else {
                        self.de.input = &[];
                    }
                }
            }

            self.len -= n;
        }

        self.next()
    }

    #[inline]
    fn fold<B, Fun>(mut self, init: B, mut f: Fun) -> B
    where
        Fun: FnMut(B, Result<T, DeserializeError>) -> B,
    {
        let mut acc = init;
        for _ in 0..self.len {
            match E::deserialize(&mut self.de) {
                Ok(item) => acc = f(acc, Ok(item)),
                Err(err) => {
                    acc = f(acc, Err(err));
                    break;
                }
            };
        }
        acc
    }
}

impl<'de, E, T, const SIZE_BYTES: usize> DoubleEndedIterator for DeIter<'de, E, T, SIZE_BYTES>
where
    E: Element + ?Sized,
    T: Deserialize<'de, E::Formula>,
{
    #[inline]
    fn next_back(&mut self) -> Option<Result<T, DeserializeError>> {
        if self.len == 0 {
            return None;
        }

        match self.read_at(self.len - 1) {
            Ok(item) => {
                self.len -= 1;
                Some(Ok(item))
            }
            Err(err) => {
                self.len = 0;
                Some(Err(err))
            }
        }
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Result<T, DeserializeError>> {
        if n > 0 {
            if n >= self.len {
                self.len = 0;
                return None;
            }
            self.len -= n;
        }
        self.next_back()
    }

    #[inline]
    fn rfold<B, Fun>(self, init: B, mut f: Fun) -> B
    where
        Fun: FnMut(B, Result<T, DeserializeError>) -> B,
    {
        let mut acc = init;
        for idx in (0..self.len).rev() {
            match self.read_at(idx) {
                Ok(item) => acc = f(acc, Ok(item)),
                Err(err) => {
                    acc = f(acc, Err(err));
                    break;
                }
            };
        }
        acc
    }
}

impl<'de, F, T, const SIZE_BYTES: usize> ExactSizeIterator for DeIter<'de, F, T, SIZE_BYTES>
where
    F: Formula + ?Sized,
    T: Deserialize<'de, F>,
{
    #[inline]
    fn len(&self) -> usize {
        self.len
    }
}

impl<'de, E, T, const SIZE_BYTES: usize> FusedIterator for DeIter<'de, E, T, SIZE_BYTES>
where
    E: Element + ?Sized,
    T: Deserialize<'de, E::Formula>,
{
}

/// Deserializes value from the input.
/// Returns deserialized value.
/// Input slice must be exactly the length returned by serialization function.
///
/// To use with streams where end of input is not known in advance use [`pack`] and [`unpack`] functions.
///
/// # Errors
///
/// Returns `DeserializeError` if deserialization fails.
///
/// [`pack`]: crate::pack
/// [`unpack`]: crate::unpack
#[inline]
pub fn deserialize<'de, E, T, const SIZE_BYTES: usize>(
    input: &'de [u8],
) -> Result<T, DeserializeError>
where
    E: Element + ?Sized,
    T: Deserialize<'de, E::Formula>,
{
    let mut de = DeserializerImpl::<SIZE_BYTES>::new(input);
    E::deserialize(&mut de)
}

/// Deserializes value from the input.
/// Updates value in-place.
/// Input slice must be exactly the length returned by serialization function.
///
/// To use with streams where end of input is not known in advance use [`pack`] and [`unpack`] functions.
///
/// # Errors
///
/// Returns `DeserializeError` if deserialization fails.
///
/// [`pack`]: crate::pack
/// [`unpack`]: crate::unpack
#[inline]
pub fn deserialize_in_place<'de, E, T, const SIZE_BYTES: usize>(
    place: &mut T,
    input: &'de [u8],
) -> Result<(), DeserializeError>
where
    E: Element + ?Sized,
    T: Deserialize<'de, E::Formula> + ?Sized,
{
    let mut de = DeserializerImpl::<SIZE_BYTES>::new(input);
    simple_try!(E::deserialize_in_place(place, &mut de));
    Ok(())
}

#[cold]
fn too_large_error<const SIZE_BYTES: usize>(input: &[u8; SIZE_BYTES]) -> DeserializeError {
    DeserializeError::TooLarge(u128::from_le_bytes({
        let mut arr = [0u8; 16];
        arr[..SIZE_BYTES].copy_from_slice(input);
        arr
    }))
}

#[inline]
pub fn read_usize<const SIZE_BYTES: usize>(
    input: &[u8; SIZE_BYTES],
) -> Result<usize, DeserializeError> {
    const {
        assert!(SIZE_BYTES > 0 && SIZE_BYTES <= 16);
    }

    const LEN: usize = size_of::<usize>();

    match () {
        () if SIZE_BYTES > LEN => {
            let zero_tail = input[LEN..] == [0u8; SIZE_BYTES][LEN..];
            if !zero_tail {
                return Err(too_large_error(input));
            }
            let mut bytes = [0u8; LEN];
            bytes.copy_from_slice(&input[..LEN]);
            Ok(usize::from_le_bytes(bytes))
        }
        () if SIZE_BYTES < LEN => {
            let mut bytes = [0u8; LEN];
            bytes[..SIZE_BYTES].copy_from_slice(input);
            Ok(usize::from_le_bytes(bytes))
        }
        () => {
            let mut bytes = [0u8; LEN];
            bytes.copy_from_slice(input);
            Ok(usize::from_le_bytes(bytes))
        }
    }
}

macro_rules! fixed_size_module {
    ($(#[$meta:meta])* $vis:vis mod $module:ident { $size_bytes:literal }) => {
        $(#[$meta])*
         $vis mod $module {
            use super::*;

            /// Deserializes value from the input.
            /// Returns deserialized value.
            ///
            /// # Errors
            ///
            /// Returns `DeserializeError` if deserialization fails.
            #[inline]
            pub fn deserialize<'de, E, T>(
                input: &'de [u8],
            ) -> Result<T, DeserializeError>
            where
                E: Element + ?Sized,
                T: Deserialize<'de, E::Formula>,
            {
                let mut de = DeserializerImpl::<$size_bytes>::new(input);
                E::deserialize(&mut de)
            }

            /// Deserializes value from the input.
            /// Updates value in-place.
            ///
            /// # Errors
            ///
            /// Returns `DeserializeError` if deserialization fails.
            #[inline]
            pub fn deserialize_in_place<'de, E, T>(
                place: &mut T,
                input: &'de [u8],
            ) -> Result<(), DeserializeError>
            where
                E: Element + ?Sized,
                T: Deserialize<'de, E::Formula> + ?Sized,
            {
                let mut de = DeserializerImpl::<$size_bytes>::new(input);
                simple_try!(E::deserialize_in_place(place, &mut de));
                Ok(())
            }
        }
    };
}

fixed_size_module! {
    /// Deserialization functions for small data.
    ///
    /// They use only 1 byte to encode sizes and indirection,
    /// so max size is 255 bytes and max length of sequences is 255 elements,
    /// even if elements are zero-sized.
    pub mod small { 1 }
}

fixed_size_module! {
    /// Deserialization functions for medium data.
    ///
    /// They use only 2 bytes to encode sizes and indirection,
    /// so max size is 65535 bytes and max length of sequences is 65535 elements,
    /// even if elements are zero-sized.
    pub mod medium { 2 }
}

fixed_size_module! {
    /// Deserialization functions for large data.
    ///
    /// They use only 4 bytes to encode sizes and indirection,
    /// so max size is 4294967295 bytes and max length of sequences is 4294967295 elements,
    /// even if elements are zero-sized.
    pub mod large { 4 }
}

fixed_size_module! {
    /// Deserialization functions for huge data.
    ///
    /// They use 8 bytes to encode sizes and indirection,
    /// so max size is 18446744073709551615 bytes and max length of
    /// sequences is 18446744073709551615 elements, even if elements are zero-sized.
    pub mod huge { 8 }
}

fixed_size_module! {
    /// Deserialization functions for humongous data.
    ///
    /// They use 16 bytes to encode sizes and indirection,
    /// so max size is 340282366920938463463374607431768211455 bytes and max length of
    /// sequences is 340282366920938463463374607431768211455 elements, even if elements are zero-sized.
    pub mod humongous { 16 }
}
