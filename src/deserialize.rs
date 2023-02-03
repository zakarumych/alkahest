use core::mem::size_of;

use crate::{
    formula::{Formula, UnsizedFormula},
    size::{FixedIsizeType, FixedUsize, FixedUsizeType},
};

#[derive(Debug)]
pub enum DeserializeError {
    /// Indicates that input buffer is smaller than
    /// expected value length.
    OutOfBounds,

    /// Relative address points behind itself.
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
pub trait Deserialize<'de, F: UnsizedFormula + ?Sized> {
    /// Deserializes value from bytes slice.
    /// Returns deserialized value and the number of bytes consumed from
    /// the and of input.
    ///
    /// The value appears at the end of the slice.
    /// And referenced values are addressed from the beginning of the slice.
    fn deserialize(len: usize, input: &'de [u8]) -> Result<Self, DeserializeError>
    where
        Self: Sized;

    /// Deserializes value in-place from bytes slice.
    /// Overwrites `self` with data from the `input`.
    ///
    /// The value appears at the end of the slice.
    /// And referenced values are addressed from the beginning of the slice.
    fn deserialize_in_place(
        &mut self,
        len: usize,
        input: &'de [u8],
    ) -> Result<(), DeserializeError>;
}

#[must_use]
pub struct Deserializer<'de> {
    /// Input buffer sub-slice usable for deserialization.
    input: &'de [u8],
    len: usize,
}

impl<'de> Deserializer<'de> {
    pub fn new(len: usize, input: &'de [u8]) -> Result<Self, DeserializeError> {
        if len > input.len() {
            return Err(DeserializeError::OutOfBounds);
        }
        Ok(Deserializer { input, len })
    }

    pub fn deserialize<F, T>(&mut self, len: usize) -> Result<T, DeserializeError>
    where
        F: UnsizedFormula + ?Sized,
        T: Deserialize<'de, F>,
    {
        if len > self.len {
            return Err(DeserializeError::OutOfBounds);
        }
        let value = T::deserialize(len, self.input)?;
        let end = self.input.len() - len;
        self.input = &self.input[..end];
        self.len -= len;
        Ok(value)
    }

    pub fn deserialize_self<T>(&mut self) -> Result<T, DeserializeError>
    where
        T: Deserialize<'de, T> + Formula,
    {
        self.deserialize::<T, T>(T::SIZE)
    }

    pub fn deserialize_sized<F, T>(&mut self) -> Result<T, DeserializeError>
    where
        F: Formula + ?Sized,
        T: Deserialize<'de, F>,
    {
        self.deserialize::<F, T>(F::SIZE)
    }

    pub fn deserialize_rest<F, T>(&mut self) -> Result<T, DeserializeError>
    where
        F: UnsizedFormula + ?Sized,
        T: Deserialize<'de, F>,
    {
        self.deserialize::<F, T>(self.len)
    }

    pub fn deserialize_in_place<F, T>(
        &mut self,
        place: &mut T,
        len: usize,
    ) -> Result<(), DeserializeError>
    where
        F: UnsizedFormula + ?Sized,
        T: Deserialize<'de, F> + ?Sized,
    {
        if len > self.len {
            return Err(DeserializeError::OutOfBounds);
        }

        T::deserialize_in_place(place, len, self.input)?;
        self.consume(len)?;

        Ok(())
    }

    pub fn deserialize_in_place_sized<F, T>(
        &mut self,
        place: &mut T,
    ) -> Result<(), DeserializeError>
    where
        F: Formula + ?Sized,
        T: Deserialize<'de, F>,
    {
        self.deserialize_in_place::<F, T>(place, F::SIZE)
    }

    pub fn deserialize_in_place_self<T>(&mut self, place: &mut T) -> Result<(), DeserializeError>
    where
        T: Formula + Deserialize<'de, T>,
    {
        self.deserialize_in_place::<T, T>(place, T::SIZE)
    }

    pub fn deserialize_in_place_rest<F, T>(&mut self, place: &mut T) -> Result<(), DeserializeError>
    where
        F: UnsizedFormula + ?Sized,
        T: Deserialize<'de, F>,
    {
        self.deserialize_in_place::<F, T>(place, self.len)
    }

    #[inline]
    pub fn consume(&mut self, len: usize) -> Result<(), DeserializeError> {
        if len > self.len {
            Err(DeserializeError::WrongLength)
        } else {
            let end = self.input.len() - len;
            self.input = &self.input[..end];
            self.len -= len;
            Ok(())
        }
    }

    #[inline]
    pub fn consume_tail(&mut self) {
        let _ = self.consume(self.len);
    }

    pub fn finish_expected(self) {
        debug_assert_eq!(self.len, 0, "All bytes should be consumed");
    }

    pub fn finish_checked(self) -> Result<(), DeserializeError> {
        if self.len == 0 {
            Ok(())
        } else {
            Err(DeserializeError::WrongLength)
        }
    }
}

pub fn deserialize<'de, F, T>(input: &'de [u8]) -> Result<(T, usize), DeserializeError>
where
    F: UnsizedFormula + ?Sized,
    T: Deserialize<'de, F>,
{
    const FIELD_SIZE: usize = size_of::<FixedUsize>();
    const HEADER_SIZE: usize = FIELD_SIZE * 2;

    if input.len() < HEADER_SIZE {
        return Err(DeserializeError::OutOfBounds);
    }

    let mut de = Deserializer::new(HEADER_SIZE, &input[..HEADER_SIZE])?;

    let size = de.deserialize::<FixedUsize, FixedUsize>(FIELD_SIZE)?;
    let address = de.deserialize::<FixedUsize, FixedUsize>(FIELD_SIZE)?;
    de.finish_expected();

    if size > address {
        return Err(DeserializeError::OutOfBounds);
    }

    if usize::from(address) > input.len() {
        return Err(DeserializeError::OutOfBounds);
    }

    let mut de = Deserializer::new(size.into(), &input[..usize::from(address)])?;
    let value = de.deserialize::<F, T>(size.into())?;
    de.finish_expected();

    Ok((value, address.into()))
}
