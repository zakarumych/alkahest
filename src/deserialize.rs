use crate::{schema::Schema, size::FixedUsizeType};

pub enum DeserializeError {
    /// The input is smaller than required to deserialize value.
    OutOfBounds,

    /// Size value exceeds the maximum size for current architecture.
    InvalidSize(FixedUsizeType),
}

/// Trait for types that can be deserialized
/// from raw bytes with specified `S: `[`Schema`].
pub trait Deserialize<'a, S: Schema + ?Sized> {
    /// Deserializes value from bytes slice.
    ///
    /// The value metadata appears at the end of the slice.
    /// And payload can be anywhere in the slice.
    /// Metadata will contain offset for the payload if there any.
    fn deserialize(input: &'a [u8]) -> Result<(Self, usize), DeserializeError>
    where
        Self: Sized;

    /// Deserializes value in-place from bytes slice.
    /// Overwrites `self` with data from the `input`.
    ///
    /// The value metadata appears at the end of the slice.
    /// And payload can be anywhere in the slice.
    /// Metadata will contain offset for the payload if there any.
    fn deserialize_in_place(&mut self, input: &'a [u8]) -> Result<usize, DeserializeError>;
}

pub struct Deserializer<'a> {
    /// Input buffer sub-slice usable for deserialization.
    input: &'a [u8],
    read: usize,
}

impl<'a> Deserializer<'a> {
    pub fn new(input: &'a [u8]) -> Self {
        Deserializer { input, read: 0 }
    }

    pub fn deserialize<T, S>(&mut self) -> Result<T, DeserializeError>
    where
        S: Schema + ?Sized,
        T: Deserialize<'a, S>,
    {
        let (value, size) = T::deserialize(self.input)?;
        let end = self.input.len() - size;
        self.input = &self.input[..end];
        self.read += size;
        Ok(value)
    }

    pub fn deserialize_in_place<T, S>(&mut self, place: &mut T) -> Result<(), DeserializeError>
    where
        S: Schema + ?Sized,
        T: Deserialize<'a, S>,
    {
        let size = T::deserialize_in_place(place, self.input)?;
        let end = self.input.len() - size;
        self.input = &self.input[..end];
        self.read += size;
        Ok(())
    }

    pub fn end(self) -> usize {
        self.read
    }
}
