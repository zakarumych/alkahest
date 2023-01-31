use core::mem::size_of;

use crate::{
    schema::{Schema, SizedSchema},
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
/// from raw bytes with specified `S: `[`Schema`].
pub trait Deserialize<'a, S: Schema + ?Sized> {
    /// Deserializes value from bytes slice.
    /// Returns deserialized value and the number of bytes consumed from
    /// the and of input.
    ///
    /// The value appears at the end of the slice.
    /// And referenced values are addressed from the beginning of the slice.
    fn deserialize(len: usize, input: &'a [u8]) -> Result<Self, DeserializeError>
    where
        Self: Sized;

    /// Deserializes value in-place from bytes slice.
    /// Overwrites `self` with data from the `input`.
    ///
    /// The value appears at the end of the slice.
    /// And referenced values are addressed from the beginning of the slice.
    fn deserialize_in_place(&mut self, len: usize, input: &'a [u8])
        -> Result<(), DeserializeError>;
}

#[must_use]
pub struct Deserializer<'a> {
    /// Input buffer sub-slice usable for deserialization.
    input: &'a [u8],
    read: usize,
}

impl<'a> Deserializer<'a> {
    pub fn new(input: &'a [u8]) -> Self {
        Deserializer { input, read: 0 }
    }

    pub fn deserialize<S, T>(&mut self, len: usize) -> Result<T, DeserializeError>
    where
        S: Schema + ?Sized,
        T: Deserialize<'a, S>,
    {
        let value = T::deserialize(len, self.input)?;
        let end = self.input.len() - len;
        self.input = &self.input[..end];
        self.read += len;
        Ok(value)
    }

    pub fn deserialize_self<T>(&mut self) -> Result<T, DeserializeError>
    where
        T: Deserialize<'a, T> + SizedSchema,
    {
        self.deserialize::<T, T>(T::SIZE)
    }

    pub fn deserialize_sized<S, T>(&mut self) -> Result<T, DeserializeError>
    where
        S: SizedSchema + ?Sized,
        T: Deserialize<'a, S>,
    {
        self.deserialize::<S, T>(S::SIZE)
    }

    pub fn deserialize_in_place<S, T>(
        &mut self,
        place: &mut T,
        len: usize,
    ) -> Result<(), DeserializeError>
    where
        S: Schema + ?Sized,
        T: Deserialize<'a, S>,
    {
        T::deserialize_in_place(place, len, self.input)?;
        let end = self.input.len() - len;
        self.input = &self.input[..end];
        self.read += len;
        Ok(())
    }

    pub fn deserialize_in_place_sized<S, T>(
        &mut self,
        place: &mut T,
    ) -> Result<(), DeserializeError>
    where
        S: SizedSchema + ?Sized,
        T: Deserialize<'a, S>,
    {
        self.deserialize_in_place::<S, T>(place, S::SIZE)
    }

    pub fn deserialize_in_place_self<T>(&mut self, place: &mut T) -> Result<(), DeserializeError>
    where
        T: SizedSchema + Deserialize<'a, T>,
    {
        self.deserialize_in_place::<T, T>(place, T::SIZE)
    }

    #[must_use]
    pub fn read(&self) -> usize {
        self.read
    }

    #[must_use]
    pub fn finish(self) -> usize {
        self.read
    }
}

pub fn deserialize<'a, S, T>(input: &'a [u8]) -> Result<(T, usize), DeserializeError>
where
    S: Schema + ?Sized,
    T: Deserialize<'a, S>,
{
    const FIELD_SIZE: usize = size_of::<FixedUsize>();
    const HEADER_SIZE: usize = FIELD_SIZE * 2;

    if input.len() < HEADER_SIZE {
        return Err(DeserializeError::OutOfBounds);
    }

    let mut de = Deserializer::new(&input[..HEADER_SIZE]);

    let size = de.deserialize::<FixedUsize, FixedUsize>(FIELD_SIZE)?;
    let address = de.deserialize::<FixedUsize, FixedUsize>(FIELD_SIZE)?;

    if size > address {
        return Err(DeserializeError::OutOfBounds);
    }

    if usize::from(address) > input.len() {
        return Err(DeserializeError::OutOfBounds);
    }

    let mut de = Deserializer::new(&input[..usize::from(address)]);
    let value = de.deserialize::<S, T>(size.into())?;
    Ok((value, address.into()))
}
