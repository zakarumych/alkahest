use core::mem::size_of;

use crate::{
    formula::{Formula, UnsizedFormula},
    size::{FixedUsize, FixedUsizeType},
};

#[derive(Debug)]
pub enum DeserializeError {
    /// The input is smaller than required to deserialize value.
    OutOfBounds,

    /// Serialized value has unexpected length.
    WrongLength,

    /// Size value exceeds the maximum size for current architecture.
    InvalidSize(FixedUsizeType),
}

/// Trait for types that can be deserialized
/// from raw bytes with specified `S: `[`Formula`].
pub trait Deserialize<'de, S: UnsizedFormula + ?Sized> {
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
    pub fn new(len: usize, input: &'de [u8]) -> Self {
        Deserializer { input, len }
    }

    pub fn deserialize<S, T>(&mut self, len: usize) -> Result<T, DeserializeError>
    where
        S: UnsizedFormula + ?Sized,
        T: Deserialize<'de, S>,
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

    pub fn deserialize_sized<S, T>(&mut self) -> Result<T, DeserializeError>
    where
        S: Formula + ?Sized,
        T: Deserialize<'de, S>,
    {
        self.deserialize::<S, T>(S::SIZE)
    }

    pub fn deserialize_rest<S, T>(&mut self) -> Result<T, DeserializeError>
    where
        S: UnsizedFormula + ?Sized,
        T: Deserialize<'de, S>,
    {
        self.deserialize::<S, T>(self.len)
    }

    pub fn deserialize_in_place<S, T>(
        &mut self,
        place: &mut T,
        len: usize,
    ) -> Result<(), DeserializeError>
    where
        S: UnsizedFormula + ?Sized,
        T: Deserialize<'de, S>,
    {
        if len > self.len {
            return Err(DeserializeError::OutOfBounds);
        }

        T::deserialize_in_place(place, len, self.input)?;
        let end = self.input.len() - len;
        self.input = &self.input[..end];
        self.len -= len;

        Ok(())
    }

    pub fn deserialize_in_place_sized<S, T>(
        &mut self,
        place: &mut T,
    ) -> Result<(), DeserializeError>
    where
        S: Formula + ?Sized,
        T: Deserialize<'de, S>,
    {
        self.deserialize_in_place::<S, T>(place, S::SIZE)
    }

    pub fn deserialize_in_place_self<T>(&mut self, place: &mut T) -> Result<(), DeserializeError>
    where
        T: Formula + Deserialize<'de, T>,
    {
        self.deserialize_in_place::<T, T>(place, T::SIZE)
    }

    pub fn deserialize_in_place_rest<S, T>(&mut self, place: &mut T) -> Result<(), DeserializeError>
    where
        S: UnsizedFormula + ?Sized,
        T: Deserialize<'de, S>,
    {
        self.deserialize_in_place::<S, T>(place, self.len)
    }

    pub fn consume_tail(&mut self) {
        self.len = 0;
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

pub fn deserialize<'de, S, T>(input: &'de [u8]) -> Result<(T, usize), DeserializeError>
where
    S: UnsizedFormula + ?Sized,
    T: Deserialize<'de, S>,
{
    const FIELD_SIZE: usize = size_of::<FixedUsize>();
    const HEADER_SIZE: usize = FIELD_SIZE * 2;

    if input.len() < HEADER_SIZE {
        return Err(DeserializeError::OutOfBounds);
    }

    let mut de = Deserializer::new(HEADER_SIZE, &input[..HEADER_SIZE]);

    let size = de.deserialize::<FixedUsize, FixedUsize>(FIELD_SIZE)?;
    let address = de.deserialize::<FixedUsize, FixedUsize>(FIELD_SIZE)?;
    de.finish_expected();

    if size > address {
        return Err(DeserializeError::OutOfBounds);
    }

    if usize::from(address) > input.len() {
        return Err(DeserializeError::OutOfBounds);
    }

    let mut de = Deserializer::new(size.into(), &input[..usize::from(address)]);
    let value = de.deserialize::<S, T>(size.into())?;
    de.finish_expected();

    Ok((value, address.into()))
}
