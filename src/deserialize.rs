pub enum DeserializeError {
    /// The input is smaller than required to deserialize value.
    OutOfBounds,
}

/// Trait for types that can be deserialized
/// from raw bytes with specified `S: `[`Schema`].
pub trait Deserialize<S> {
    /// Deserializes value from bytes slice.
    ///
    /// The value metadata appears at the end of the slice.
    /// And payload can be anywhere in the slice.
    /// Metadata will contain offset for the payload if there any.
    fn deserialize(input: &[u8]) -> Result<Self, DeserializeError>
    where
        Self: Sized;

    /// Deserializes value in-place from bytes slice.
    /// Overwrites `self` with data from the `input`.
    ///
    /// The value metadata appears at the end of the slice.
    /// And payload can be anywhere in the slice.
    /// Metadata will contain offset for the payload if there any.
    fn deserialize_in_place(&mut self, input: &[u8]) -> Result<(), DeserializeError>;
}
