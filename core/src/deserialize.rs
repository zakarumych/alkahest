use core::str::Utf8Error;

use crate::{
    element::stack_size,
    formula::{Formula, SizeBound},
};

#[inline]
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
    InvalidUsize(u128),

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

pub trait Deserializer<'de> {
    fn read_bytes(&mut self, len: usize) -> Result<&'de [u8], DeserializeError>;

    fn read_byte(&mut self) -> Result<u8, DeserializeError>;

    fn read_byte_array<const N: usize>(&mut self) -> Result<[u8; N], DeserializeError>;

    fn read_usize(&mut self) -> Result<usize, DeserializeError>;

    fn read_direct<F, T>(&mut self) -> Result<T, DeserializeError>
    where
        F: Formula + ?Sized,
        T: Deserialize<'de, F>;

    /// Reads and deserializes field from the input buffer in-place.
    ///
    /// # Errors
    ///
    /// Returns `DeserializeError` if deserialization fails.
    fn read_direct_in_place<F, T>(&mut self, place: &mut T) -> Result<(), DeserializeError>
    where
        F: Formula + ?Sized,
        T: Deserialize<'de, F> + ?Sized;

    fn read_indirect<F, T>(&mut self) -> Result<T, DeserializeError>
    where
        F: Formula + ?Sized,
        T: Deserialize<'de, F>;

    /// Reads and deserializes field from the input buffer in-place.
    ///
    /// # Errors
    ///
    /// Returns `DeserializeError` if deserialization fails.
    fn read_indirect_in_place<F, T>(&mut self, place: &mut T) -> Result<(), DeserializeError>
    where
        F: Formula + ?Sized,
        T: Deserialize<'de, F> + ?Sized;

    // /// Converts deserializer into iterator over deserialized values with
    // /// specified formula.
    // /// The formula must be sized and size must match.
    // ///
    // /// # Panics
    // ///
    // /// Panics if formula is not sized.
    // #[inline]
    // fn into_array_iter<F, T>(self, len: usize) -> DeIter<'de, F, T, SIZE_BYTES>
    // where
    //     F: Formula + ?Sized,
    //     T: Deserialize<'de, F>,
    //     Self: Sized,
    // {
    //     DeIter {
    //         de: self,
    //         marker: PhantomData,
    //         len,
    //     }
    // }
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

/// Trait for types that can be deserialized
/// from raw bytes with specified `F: `[`Formula`].
pub trait DeserializeInPlace<'de, F: Formula + ?Sized> {
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
pub struct DeserializerImpl<'de, const SIZE_BYTES: u8> {
    /// Input buffer sub-slice usable for deserialization.
    input: &'de [u8],

    #[cfg(debug_assertions)]
    debug_exhausted: bool,
}

impl<'de, const SIZE_BYTES: u8> DeserializerImpl<'de, SIZE_BYTES> {
    /// Creates new deserializer from input buffer.
    ///
    /// # Errors
    ///
    /// Returns `DeserializeError::OutOfBounds` if
    /// `stack` is greater than `input.len()`.
    #[inline]
    pub const fn new(input: &'de [u8]) -> Self {
        DeserializerImpl {
            input,
            #[cfg(debug_assertions)]
            debug_exhausted: false,
        }
    }

    /// Reads and deserializes reference from the input buffer.
    ///
    /// # Errors
    ///
    /// Returns `DeserializeError` if reference is out of bounds
    /// or has address larger that self.
    #[inline]
    fn indirection<F>(&mut self) -> Result<DeserializerImpl<'de, SIZE_BYTES>, DeserializeError>
    where
        F: Formula + ?Sized,
    {
        let address = self.read_usize()?;
        Ok(DeserializerImpl::new(&self.input[..address]))
    }

    #[cfg(debug_assertions)]
    #[inline]
    fn debug_validate(&self) {
        assert!(!self.debug_exhausted, "Deserializer used after exhaustion");
    }

    fn skip_padding<F>(&mut self, new_len: &mut usize)
    where
        F: Formula + ?Sized,
    {
        match stack_size::<F, SIZE_BYTES>() {
            SizeBound::Bounded(max_stack) => {
                debug_assert!(*new_len >= self.input.len() - max_stack);

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

impl<'de, const SIZE_BYTES: u8> Deserializer<'de> for DeserializerImpl<'de, SIZE_BYTES> {
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
    fn read_byte_array<const N: usize>(&mut self) -> Result<[u8; N], DeserializeError> {
        #[cfg(debug_assertions)]
        self.debug_validate();

        if N > self.input.len() {
            return cold_err(DeserializeError::WrongLength);
        }
        let at = self.input.len() - N;

        let (head, tail) = self.input.split_at(at);
        self.input = head;

        let mut array = [0; N];
        array.copy_from_slice(tail);
        Ok(array)
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

        let len = usize::from(SIZE_BYTES);
        let max_len = size_of::<usize>();

        let input = self.read_bytes(len)?;
        let mut bytes = [0u8; size_of::<usize>()];

        if max_len < len {
            // If SIZE_BYTES exceeds usize, ensure that the extra bytes are zero.
            let all_zero = input[max_len..] == [0u8; 256][..len - max_len];
            if !all_zero {
                return Err(DeserializeError::InvalidUsize(u128::from_le_bytes({
                    debug_assert!(input.len() <= 16);
                    let mut arr = [0u8; 16];
                    arr[..input.len()].copy_from_slice(input);
                    arr
                })));
            }

            bytes[..max_len].copy_from_slice(&input[..max_len]);
        } else {
            bytes[..len].copy_from_slice(&input[..len]);
        }

        Ok(usize::from_le_bytes(bytes))
    }

    /// Reads and deserializes field from the input buffer.
    /// Advances the input buffer.
    ///
    /// # Errors
    ///
    /// Returns `DeserializeError` if deserialization fails.
    #[inline]
    fn read_direct<F, T>(&mut self) -> Result<T, DeserializeError>
    where
        F: Formula + ?Sized,
        T: Deserialize<'de, F>,
    {
        #[cfg(debug_assertions)]
        self.debug_validate();

        let mut new_len: usize = 0;
        let value = <T as Deserialize<'de, F>>::deserialize(
            TrackingDeserializerImpl::<SIZE_BYTES>::new(self.input, &mut new_len),
        )?;

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
    fn read_direct_in_place<F, T>(&mut self, place: &mut T) -> Result<(), DeserializeError>
    where
        F: Formula + ?Sized,
        T: Deserialize<'de, F> + ?Sized,
    {
        #[cfg(debug_assertions)]
        self.debug_validate();

        let mut new_len: usize = 0;
        <T as Deserialize<'de, F>>::deserialize_in_place(
            place,
            TrackingDeserializerImpl::<SIZE_BYTES>::new(self.input, &mut new_len),
        )?;

        self.skip_padding::<F>(&mut new_len);

        self.input = &self.input[..new_len];
        Ok(())
    }

    #[inline]
    fn read_indirect<F, T>(&mut self) -> Result<T, DeserializeError>
    where
        F: Formula + ?Sized,
        T: Deserialize<'de, F>,
    {
        #[cfg(debug_assertions)]
        self.debug_validate();

        let address = self.read_usize()?;

        if address > self.input.len() {
            return cold_err(DeserializeError::WrongAddress);
        }

        let de = DeserializerImpl::<SIZE_BYTES>::new(&self.input[..address]);
        <T as Deserialize<'de, F>>::deserialize(de)
    }

    #[inline]
    fn read_indirect_in_place<F, T>(&mut self, place: &mut T) -> Result<(), DeserializeError>
    where
        F: Formula + ?Sized,
        T: Deserialize<'de, F> + ?Sized,
    {
        #[cfg(debug_assertions)]
        self.debug_validate();

        let address = self.read_usize()?;

        if address > self.input.len() {
            return cold_err(DeserializeError::WrongAddress);
        }

        let de = DeserializerImpl::<SIZE_BYTES>::new(&self.input[..address]);
        <T as Deserialize<'de, F>>::deserialize_in_place(place, de)
    }
}

/// Deserializer from raw bytes.
/// Provides methods for deserialization of values.
#[must_use = "Deserializer should be used to deserialize values"]
pub struct TrackingDeserializerImpl<'de, 'consumed, const SIZE_BYTES: u8> {
    /// Input buffer sub-slice usable for deserialization.
    inner: DeserializerImpl<'de, SIZE_BYTES>,

    rest: &'consumed mut usize,
}

impl<'de, 'consumed, const SIZE_BYTES: u8> Drop
    for TrackingDeserializerImpl<'de, 'consumed, SIZE_BYTES>
{
    fn drop(&mut self) {
        *self.rest = self.inner.input.len();
    }
}

impl<'de, 'consumed, const SIZE_BYTES: u8> TrackingDeserializerImpl<'de, 'consumed, SIZE_BYTES> {
    #[inline]
    pub const fn new(input: &'de [u8], rest: &'consumed mut usize) -> Self {
        TrackingDeserializerImpl {
            inner: DeserializerImpl::new(input),
            rest,
        }
    }
}

impl<'de, const SIZE_BYTES: u8> Deserializer<'de>
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
    fn read_byte_array<const N: usize>(&mut self) -> Result<[u8; N], DeserializeError> {
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
    fn read_direct<F, T>(&mut self) -> Result<T, DeserializeError>
    where
        F: Formula + ?Sized,
        T: Deserialize<'de, F>,
    {
        self.inner.read_direct()
    }

    /// Reads and deserializes field from the input buffer in-place.
    ///
    /// # Errors
    ///
    /// Returns `DeserializeError` if deserialization fails.
    #[inline]
    fn read_direct_in_place<F, T>(&mut self, place: &mut T) -> Result<(), DeserializeError>
    where
        F: Formula + ?Sized,
        T: Deserialize<'de, F> + ?Sized,
    {
        self.inner.read_direct_in_place(place)
    }

    /// Reads and deserializes field from the input buffer.
    /// Advances the input buffer.
    ///
    /// # Errors
    ///
    /// Returns `DeserializeError` if deserialization fails.
    #[inline]
    fn read_indirect<F, T>(&mut self) -> Result<T, DeserializeError>
    where
        F: Formula + ?Sized,
        T: Deserialize<'de, F>,
    {
        self.inner.read_indirect()
    }

    /// Reads and deserializes field from the input buffer in-place.
    ///
    /// # Errors
    ///
    /// Returns `DeserializeError` if deserialization fails.
    #[inline]
    fn read_indirect_in_place<F, T>(&mut self, place: &mut T) -> Result<(), DeserializeError>
    where
        F: Formula + ?Sized,
        T: Deserialize<'de, F> + ?Sized,
    {
        self.inner.read_indirect_in_place(place)
    }
}

// /// Iterator over deserialized values.
// #[must_use]
// pub struct DeIter<'de, F: ?Sized, T, const SIZE_BYTES: u8 = 8> {
//     de: Deserializer<'de>,
//     len: usize,
//     marker: PhantomData<fn(F) -> T>,
// }

// impl<'de, F, T, const SIZE_BYTES: u8> DeIter<'de, F, T, SIZE_BYTES>
// where
//     F: Formula + ?Sized,
//     T: Deserialize<'de, F>,
// {
//     /// Returns true if no items remains in the iterator.
//     #[must_use]
//     #[inline]
//     pub fn is_empty(&self) -> bool {
//         self.len == 0
//     }
// }

// impl<'de, F, T, const SIZE_BYTES: u8> Clone for DeIter<'de, F, T, SIZE_BYTES>
// where
//     F: ?Sized,
// {
//     #[inline]
//     fn clone(&self) -> Self {
//         DeIter {
//             de: self.de.clone(),
//             marker: PhantomData,
//             len: self.len,
//         }
//     }

//     #[inline]
//     fn clone_from(&mut self, source: &Self) {
//         self.de = source.de.clone();
//         self.len = source.len;
//     }
// }

// impl<'de, F, T, const SIZE_BYTES: u8> Iterator for DeIter<'de, F, T, SIZE_BYTES>
// where
//     F: Formula + ?Sized,
//     T: Deserialize<'de, F>,
// {
//     type Item = Result<T, DeserializeError>;

//     #[inline]
//     fn size_hint(&self) -> (usize, Option<usize>) {
//         (self.len, Some(self.len))
//     }

//     #[inline]
//     fn next(&mut self) -> Option<Result<T, DeserializeError>> {
//         if self.is_empty() {
//             return None;
//         }
//         let item = self.de.read_value::<F, T>(self.len > 1);
//         self.len -= 1;
//         Some(item)
//     }

//     #[inline]
//     fn count(self) -> usize {
//         match F::MAX_STACK_SIZE {
//             None => self.fold(0, |acc, _| acc + 1),
//             Some(_) => self.len,
//         }
//     }

//     #[inline]
//     fn nth(&mut self, n: usize) -> Option<Result<T, DeserializeError>> {
//         if n > 0 {
//             if n >= self.len {
//                 self.len = 0;
//                 return None;
//             }
//             if let Err(err) = self.de.skip_values::<F>(n) {
//                 self.len = 0;
//                 return Some(Err(err));
//             }
//             self.len -= n;
//         }
//         self.next()
//     }

//     #[inline]
//     fn fold<B, Fun>(mut self, mut init: B, mut f: Fun) -> B
//     where
//         Fun: FnMut(B, Result<T, DeserializeError>) -> B,
//     {
//         match F::MAX_STACK_SIZE {
//             None => loop {
//                 if self.de.stack < usize::from(SIZE_BYTES) {
//                     break;
//                 }

//                 let stack = match self.de.read_usize() {
//                     Ok(stack) => stack,
//                     Err(err) => {
//                         self.de.stack = 0;
//                         return f(init, cold_err(err));
//                     }
//                 };

//                 match self.de.sub(stack) {
//                     Ok(sub) => {
//                         let result = <T as Deserialize<'de, F>>::deserialize(sub);
//                         init = f(init, result);
//                     }
//                     Err(err) => {
//                         self.de.stack = 0;
//                         return f(init, cold_err(err));
//                     }
//                 }
//             },
//             Some(0) => {
//                 let sub = Deserializer::new_unchecked(0, self.de.input);
//                 for _ in 0..self.upper {
//                     let result = <T as Deserialize<'de, F>>::deserialize(sub.clone());
//                     init = f(init, result);
//                 }
//             }
//             Some(stack) => {
//                 assert_eq!(self.de.stack / stack, self.upper);

//                 for _ in 0..self.upper {
//                     let sub = Deserializer::new_unchecked(stack, self.de.input);
//                     self.de.input = &self.de.input[..self.de.input.len() - stack];

//                     let result = <T as Deserialize<'de, F>>::deserialize(sub);
//                     init = f(init, result);
//                 }
//             }
//         }
//         init
//     }
// }

// impl<'de, F, T, const SIZE_BYTES: u8> DeIter<'de, F, T, SIZE_BYTES>
// where
//     F: Formula + ?Sized,
//     T: Deserialize<'de, F>,
// {
//     const ELEMENT_SIZE: usize = F::MAX_STACK_SIZE.unwrap();
// }

// impl<'de, F, T, const SIZE_BYTES: u8> DoubleEndedIterator for DeIter<'de, F, T, SIZE_BYTES>
// where
//     F: Formula + ?Sized,
//     T: Deserialize<'de, F>,
// {
//     #[inline]
//     fn next_back(&mut self) -> Option<Result<T, DeserializeError>> {
//         if Self::is_empty(self) {
//             return None;
//         }
//         let item = self.de.read_back_value::<F, T>();
//         self.upper -= 1;
//         Some(item)
//     }

//     #[inline]
//     fn nth_back(&mut self, n: usize) -> Option<Result<T, DeserializeError>> {
//         if n > 0 {
//             if n >= self.upper {
//                 self.upper = 0;
//                 return None;
//             }
//             self.de.stack -= n * Self::ELEMENT_SIZE;
//             self.upper -= n;
//         }
//         self.next_back()
//     }

//     #[inline]
//     fn rfold<B, Fun>(self, mut init: B, mut f: Fun) -> B
//     where
//         Fun: FnMut(B, Result<T, DeserializeError>) -> B,
//     {
//         match Self::ELEMENT_SIZE {
//             0 => {
//                 let sub = Deserializer::new_unchecked(0, self.de.input);
//                 for _ in 0..self.upper {
//                     let result = <T as Deserialize<'de, F>>::deserialize(sub.clone());
//                     init = f(init, result);
//                 }
//             }
//             stack => {
//                 assert_eq!(self.de.stack / stack, self.upper);
//                 let mut end = self.de.input.len() - stack * self.upper;
//                 for _ in 0..self.upper {
//                     end += stack;
//                     let sub = Deserializer::new_unchecked(stack, &self.de.input[..end]);

//                     let result = <T as Deserialize<'de, F>>::deserialize(sub);
//                     init = f(init, result);
//                 }
//             }
//         }
//         init
//     }
// }

// impl<'de, F, T, const SIZE_BYTES: u8> ExactSizeIterator for DeIter<'de, F, T, SIZE_BYTES>
// where
//     F: Formula + ?Sized,
//     T: Deserialize<'de, F>,
// {
//     #[inline]
//     fn len(&self) -> usize {
//         self.len
//     }
// }

// impl<'de, F, T, const SIZE_BYTES: u8> FusedIterator for DeIter<'de, F, T, SIZE_BYTES>
// where
//     F: Formula + ?Sized,
//     T: Deserialize<'de, F>,
// {
// }

/// Deserializes value from the input.
/// Returns deserialized value.
///
/// # Errors
///
/// Returns `DeserializeError` if deserialization fails.
#[inline]
pub fn deserialize<'de, F, T, const SIZE_BYTES: u8>(input: &'de [u8]) -> Result<T, DeserializeError>
where
    F: Formula + ?Sized,
    T: Deserialize<'de, F>,
{
    let de = DeserializerImpl::<SIZE_BYTES>::new(input);
    let value = <T as Deserialize<'de, F>>::deserialize(de)?;

    Ok(value)
}

/// Deserializes value from the input.
/// Updates value in-place.
///
/// # Errors
///
/// Returns `DeserializeError` if deserialization fails.
#[inline]
pub fn deserialize_in_place<'de, F, T, const SIZE_BYTES: u8>(
    place: &mut T,
    input: &'de [u8],
) -> Result<(), DeserializeError>
where
    F: Formula + ?Sized,
    T: Deserialize<'de, F> + ?Sized,
{
    let de = DeserializerImpl::<SIZE_BYTES>::new(input);
    <T as Deserialize<'de, F>>::deserialize_in_place(place, de)?;

    Ok(())
}
