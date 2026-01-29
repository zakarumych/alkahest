use core::{iter::FusedIterator, str::Utf8Error};

use crate::formula::Formula;

#[inline(always)]
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

pub trait Deserializer<'de, const SIZE_BYTES: u8> {
    fn read_bytes(&mut self, len: usize) -> Result<&'de [u8], DeserializeError>;

    fn read_byte(&mut self) -> Result<u8, DeserializeError>;

    fn read_byte_array<const N: usize>(&mut self) -> Result<[u8; N], DeserializeError>;

    fn read_usize(&mut self) -> Result<usize, DeserializeError>;

    fn read_direct<F, T>(&mut self, last: bool) -> Result<T, DeserializeError>
    where
        F: Formula<SIZE_BYTES> + ?Sized,
        T: Deserialize<'de, F, SIZE_BYTES>;

    /// Reads and deserializes field from the input buffer in-place.
    ///
    /// # Errors
    ///
    /// Returns `DeserializeError` if deserialization fails.
    fn read_direct_in_place<F, T>(
        &mut self,
        place: &mut T,
        last: bool,
    ) -> Result<(), DeserializeError>
    where
        F: Formula<SIZE_BYTES> + ?Sized,
        T: Deserialize<'de, F, SIZE_BYTES> + ?Sized;

    fn read_indirect<F, T>(&mut self) -> Result<T, DeserializeError>
    where
        F: Formula<SIZE_BYTES> + ?Sized,
        T: Deserialize<'de, F, SIZE_BYTES>;

    /// Reads and deserializes field from the input buffer in-place.
    ///
    /// # Errors
    ///
    /// Returns `DeserializeError` if deserialization fails.
    fn read_indirect_in_place<F, T>(&mut self, place: &mut T) -> Result<(), DeserializeError>
    where
        F: Formula<SIZE_BYTES> + ?Sized,
        T: Deserialize<'de, F, SIZE_BYTES> + ?Sized;

    // /// Converts deserializer into iterator over deserialized values with
    // /// specified formula.
    // /// The formula must be sized and size must match.
    // ///
    // /// # Panics
    // ///
    // /// Panics if formula is not sized.
    // #[inline(always)]
    // fn into_array_iter<F, T>(self, len: usize) -> DeIter<'de, F, T, SIZE_BYTES>
    // where
    //     F: Formula<SIZE_BYTES> + ?Sized,
    //     T: Deserialize<'de, F, SIZE_BYTES>,
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
pub trait Deserialize<'de, F: Formula<SIZE_BYTES> + ?Sized, const SIZE_BYTES: u8> {
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
        D: Deserializer<'de, SIZE_BYTES>,
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
        D: Deserializer<'de, SIZE_BYTES>;
}

/// Deserializer from raw bytes.
/// Provides methods for deserialization of values.
#[must_use = "Deserializer should be used to deserialize values"]
pub struct DeserializerImpl<'de, const SIZE_BYTES: u8> {
    /// Input buffer sub-slice usable for deserialization.
    input: &'de [u8],
}

impl<'de, const SIZE_BYTES: u8> DeserializerImpl<'de, SIZE_BYTES> {
    /// Creates new deserializer from input buffer.
    ///
    /// # Errors
    ///
    /// Returns `DeserializeError::OutOfBounds` if
    /// `stack` is greater than `input.len()`.
    #[inline(always)]
    pub const fn new(input: &'de [u8]) -> Self {
        DeserializerImpl { input }
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
        F: Formula<SIZE_BYTES> + ?Sized,
    {
        let address = self.read_usize()?;
        Ok(DeserializerImpl::new(&self.input[..address]))
    }
}

impl<'de, const SIZE_BYTES: u8> Deserializer<'de, SIZE_BYTES>
    for DeserializerImpl<'de, SIZE_BYTES>
{
    /// Reads specified number of bytes from the input buffer.
    /// Returns slice of bytes.
    /// Advances the input buffer.
    ///
    /// # Errors
    ///
    /// Returns `DeserializeError` if not enough bytes on stack.
    #[inline(always)]
    fn read_bytes(&mut self, len: usize) -> Result<&'de [u8], DeserializeError> {
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
    #[inline(always)]
    fn read_byte(&mut self) -> Result<u8, DeserializeError> {
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
    #[inline(always)]
    fn read_byte_array<const N: usize>(&mut self) -> Result<[u8; N], DeserializeError> {
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
    #[inline(always)]
    fn read_usize(&mut self) -> Result<usize, DeserializeError> {
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
    #[inline(always)]
    fn read_direct<F, T>(&mut self, last: bool) -> Result<T, DeserializeError>
    where
        F: Formula<SIZE_BYTES> + ?Sized,
        T: Deserialize<'de, F, SIZE_BYTES>,
    {
        let mut new_len: usize = 0;
        let value = <T as Deserialize<'de, F, SIZE_BYTES>>::deserialize(
            TrackingDeserializerImpl::new(self.input, &mut new_len),
        )?;

        match (F::MAX_STACK_SIZE, last) {
            (Some(max_stack), false) => {
                if self.input.len() < max_stack {
                    return cold_err(DeserializeError::OutOfBounds);
                }

                debug_assert!(new_len >= self.input.len() - max_stack);
                new_len = self.input.len() - max_stack;
            }
            _ => {}
        }

        self.input = &self.input[..new_len];
        Ok(value)
    }

    /// Reads and deserializes field from the input buffer in-place.
    ///
    /// # Errors
    ///
    /// Returns `DeserializeError` if deserialization fails.
    #[inline(always)]
    fn read_direct_in_place<F, T>(
        &mut self,
        place: &mut T,
        last: bool,
    ) -> Result<(), DeserializeError>
    where
        F: Formula<SIZE_BYTES> + ?Sized,
        T: Deserialize<'de, F, SIZE_BYTES> + ?Sized,
    {
        let mut new_len: usize = 0;
        <T as Deserialize<'de, F, SIZE_BYTES>>::deserialize_in_place(
            place,
            TrackingDeserializerImpl::new(self.input, &mut new_len),
        )?;

        match (F::MAX_STACK_SIZE, last) {
            (Some(max_stack), false) => {
                if self.input.len() < max_stack {
                    return cold_err(DeserializeError::OutOfBounds);
                }

                debug_assert!(new_len >= self.input.len() - max_stack);
                new_len = self.input.len() - max_stack;
            }
            _ => {}
        }

        self.input = &self.input[..new_len];
        Ok(())
    }

    #[inline(always)]
    fn read_indirect<F, T>(&mut self) -> Result<T, DeserializeError>
    where
        F: Formula<SIZE_BYTES> + ?Sized,
        T: Deserialize<'de, F, SIZE_BYTES>,
    {
        let address = self.read_usize()?;

        let de = DeserializerImpl::new(&self.input[..address]);
        <T as Deserialize<'de, F, SIZE_BYTES>>::deserialize(de)
    }

    #[inline(always)]
    fn read_indirect_in_place<F, T>(&mut self, place: &mut T) -> Result<(), DeserializeError>
    where
        F: Formula<SIZE_BYTES> + ?Sized,
        T: Deserialize<'de, F, SIZE_BYTES> + ?Sized,
    {
        let address = self.read_usize()?;

        let de = DeserializerImpl::new(&self.input[..address]);
        <T as Deserialize<'de, F, SIZE_BYTES>>::deserialize_in_place(place, de)
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
    #[inline(always)]
    pub const fn new(input: &'de [u8], rest: &'consumed mut usize) -> Self {
        TrackingDeserializerImpl {
            inner: DeserializerImpl::new(input),
            rest,
        }
    }
}

impl<'de, const SIZE_BYTES: u8> Deserializer<'de, SIZE_BYTES>
    for TrackingDeserializerImpl<'de, '_, SIZE_BYTES>
{
    /// Reads specified number of bytes from the input buffer.
    /// Returns slice of bytes.
    /// Advances the input buffer.
    ///
    /// # Errors
    ///
    /// Returns `DeserializeError` if not enough bytes on stack.
    #[inline(always)]
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
    #[inline(always)]
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
    #[inline(always)]
    fn read_byte_array<const N: usize>(&mut self) -> Result<[u8; N], DeserializeError> {
        self.inner.read_byte_array::<N>()
    }

    /// Reads and deserializes usize from the input buffer.
    /// Advances the input buffer.
    ///
    /// # Errors
    ///
    /// Returns `DeserializeError` if deserialization fails.
    #[inline(always)]
    fn read_usize(&mut self) -> Result<usize, DeserializeError> {
        self.inner.read_usize()
    }

    /// Reads and deserializes field from the input buffer.
    /// Advances the input buffer.
    ///
    /// # Errors
    ///
    /// Returns `DeserializeError` if deserialization fails.
    #[inline(always)]
    fn read_direct<F, T>(&mut self, last: bool) -> Result<T, DeserializeError>
    where
        F: Formula<SIZE_BYTES> + ?Sized,
        T: Deserialize<'de, F, SIZE_BYTES>,
    {
        self.inner.read_direct(last)
    }

    /// Reads and deserializes field from the input buffer in-place.
    ///
    /// # Errors
    ///
    /// Returns `DeserializeError` if deserialization fails.
    #[inline(always)]
    fn read_direct_in_place<F, T>(
        &mut self,
        place: &mut T,
        last: bool,
    ) -> Result<(), DeserializeError>
    where
        F: Formula<SIZE_BYTES> + ?Sized,
        T: Deserialize<'de, F, SIZE_BYTES> + ?Sized,
    {
        self.inner.read_direct_in_place(place, last)
    }

    /// Reads and deserializes field from the input buffer.
    /// Advances the input buffer.
    ///
    /// # Errors
    ///
    /// Returns `DeserializeError` if deserialization fails.
    #[inline(always)]
    fn read_indirect<F, T>(&mut self) -> Result<T, DeserializeError>
    where
        F: Formula<SIZE_BYTES> + ?Sized,
        T: Deserialize<'de, F, SIZE_BYTES>,
    {
        self.inner.read_indirect()
    }

    /// Reads and deserializes field from the input buffer in-place.
    ///
    /// # Errors
    ///
    /// Returns `DeserializeError` if deserialization fails.
    #[inline(always)]
    fn read_indirect_in_place<F, T>(&mut self, place: &mut T) -> Result<(), DeserializeError>
    where
        F: Formula<SIZE_BYTES> + ?Sized,
        T: Deserialize<'de, F, SIZE_BYTES> + ?Sized,
    {
        self.inner.read_indirect_in_place(place)
    }
}

// /// Iterator over deserialized values.
// #[must_use]
// pub struct DeIter<'de, F: ?Sized, T, const SIZE_BYTES: u8 = 8> {
//     de: Deserializer<'de, SIZE_BYTES>,
//     len: usize,
//     marker: PhantomData<fn(F) -> T>,
// }

// impl<'de, F, T, const SIZE_BYTES: u8> DeIter<'de, F, T, SIZE_BYTES>
// where
//     F: Formula<SIZE_BYTES> + ?Sized,
//     T: Deserialize<'de, F, SIZE_BYTES>,
// {
//     /// Returns true if no items remains in the iterator.
//     #[must_use]
//     #[inline(always)]
//     pub fn is_empty(&self) -> bool {
//         self.len == 0
//     }
// }

// impl<'de, F, T, const SIZE_BYTES: u8> Clone for DeIter<'de, F, T, SIZE_BYTES>
// where
//     F: ?Sized,
// {
//     #[inline(always)]
//     fn clone(&self) -> Self {
//         DeIter {
//             de: self.de.clone(),
//             marker: PhantomData,
//             len: self.len,
//         }
//     }

//     #[inline(always)]
//     fn clone_from(&mut self, source: &Self) {
//         self.de = source.de.clone();
//         self.len = source.len;
//     }
// }

// impl<'de, F, T, const SIZE_BYTES: u8> Iterator for DeIter<'de, F, T, SIZE_BYTES>
// where
//     F: Formula<SIZE_BYTES> + ?Sized,
//     T: Deserialize<'de, F, SIZE_BYTES>,
// {
//     type Item = Result<T, DeserializeError>;

//     #[inline(always)]
//     fn size_hint(&self) -> (usize, Option<usize>) {
//         (self.len, Some(self.len))
//     }

//     #[inline(always)]
//     fn next(&mut self) -> Option<Result<T, DeserializeError>> {
//         if self.is_empty() {
//             return None;
//         }
//         let item = self.de.read_value::<F, T>(self.len > 1);
//         self.len -= 1;
//         Some(item)
//     }

//     #[inline(always)]
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
//                         let result = <T as Deserialize<'de, F, SIZE_BYTES>>::deserialize(sub);
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
//                     let result = <T as Deserialize<'de, F, SIZE_BYTES>>::deserialize(sub.clone());
//                     init = f(init, result);
//                 }
//             }
//             Some(stack) => {
//                 assert_eq!(self.de.stack / stack, self.upper);

//                 for _ in 0..self.upper {
//                     let sub = Deserializer::new_unchecked(stack, self.de.input);
//                     self.de.input = &self.de.input[..self.de.input.len() - stack];

//                     let result = <T as Deserialize<'de, F, SIZE_BYTES>>::deserialize(sub);
//                     init = f(init, result);
//                 }
//             }
//         }
//         init
//     }
// }

// impl<'de, F, T, const SIZE_BYTES: u8> DeIter<'de, F, T, SIZE_BYTES>
// where
//     F: Formula<SIZE_BYTES> + ?Sized,
//     T: Deserialize<'de, F, SIZE_BYTES>,
// {
//     const ELEMENT_SIZE: usize = F::MAX_STACK_SIZE.unwrap();
// }

// impl<'de, F, T, const SIZE_BYTES: u8> DoubleEndedIterator for DeIter<'de, F, T, SIZE_BYTES>
// where
//     F: Formula<SIZE_BYTES> + ?Sized,
//     T: Deserialize<'de, F, SIZE_BYTES>,
// {
//     #[inline(always)]
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
//                     let result = <T as Deserialize<'de, F, SIZE_BYTES>>::deserialize(sub.clone());
//                     init = f(init, result);
//                 }
//             }
//             stack => {
//                 assert_eq!(self.de.stack / stack, self.upper);
//                 let mut end = self.de.input.len() - stack * self.upper;
//                 for _ in 0..self.upper {
//                     end += stack;
//                     let sub = Deserializer::new_unchecked(stack, &self.de.input[..end]);

//                     let result = <T as Deserialize<'de, F, SIZE_BYTES>>::deserialize(sub);
//                     init = f(init, result);
//                 }
//             }
//         }
//         init
//     }
// }

// impl<'de, F, T, const SIZE_BYTES: u8> ExactSizeIterator for DeIter<'de, F, T, SIZE_BYTES>
// where
//     F: Formula<SIZE_BYTES> + ?Sized,
//     T: Deserialize<'de, F, SIZE_BYTES>,
// {
//     #[inline(always)]
//     fn len(&self) -> usize {
//         self.len
//     }
// }

// impl<'de, F, T, const SIZE_BYTES: u8> FusedIterator for DeIter<'de, F, T, SIZE_BYTES>
// where
//     F: Formula<SIZE_BYTES> + ?Sized,
//     T: Deserialize<'de, F, SIZE_BYTES>,
// {
// }

const fn assert_size_or_heapless<F, const SIZE_BYTES: u8>()
where
    F: Formula<SIZE_BYTES> + ?Sized,
{
    assert!(
        F::HEAPLESS || F::MAX_STACK_SIZE.is_some(),
        "The value must be either sized or heap-less"
    );
}

/// Deserializes value from the input.
/// Returns deserialized value.
///
/// # Errors
///
/// Returns `DeserializeError` if deserialization fails.
#[inline(always)]
pub fn deserialize<'de, F, T, const SIZE_BYTES: u8>(input: &'de [u8]) -> Result<T, DeserializeError>
where
    F: Formula<SIZE_BYTES> + ?Sized,
    T: Deserialize<'de, F, SIZE_BYTES>,
{
    let de = DeserializerImpl::new(input);
    let value = <T as Deserialize<'de, F, SIZE_BYTES>>::deserialize(de)?;

    Ok(value)
}

/// Deserializes value from the input.
/// Updates value in-place.
///
/// # Errors
///
/// Returns `DeserializeError` if deserialization fails.
#[inline(always)]
pub fn deserialize_in_place<'de, F, T, const SIZE_BYTES: u8>(
    place: &mut T,
    input: &'de [u8],
) -> Result<(), DeserializeError>
where
    F: Formula<SIZE_BYTES> + ?Sized,
    T: Deserialize<'de, F, SIZE_BYTES> + ?Sized,
{
    let de = DeserializerImpl::new(input);
    <T as Deserialize<'de, F, SIZE_BYTES>>::deserialize_in_place(place, de)?;

    Ok(())
}
