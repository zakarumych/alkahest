use crate::{
    deserialize::{Deserialize, DeserializeError, Deserializer},
    formula::{ExactSize, Formula},
    serialize::{Serialize, Serializer, Sizes},
};

pub type Never = core::convert::Infallible;

impl Formula for Never {
    type StackSize<const SIZE_BYTES: u8> = ExactSize<0>;
    type HeapSize<const SIZE_BYTES: u8> = ExactSize<0>;

    const INHABITED: bool = false;
}

/// Never can be serialized with any formula because it never happens.
/// This allows the following code to compile:
///
/// ```
/// #[derive(Formula)]
/// enum F {
///   A(u8),
///   B(u16),
/// }
///
/// #[derive(Serialize)]
/// #[alkahest(F)]
/// enum S {
///   A(u8),
///   B(Never),
/// }
/// ```
///
/// Since `S::B` may never be constructed, the `A` variant is the only one to be serialized.
impl<F: ?Sized> Serialize<F> for Never {
    #[inline(always)]
    fn serialize<S>(&self, _serializer: S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        match *self {}
    }

    #[inline(always)]
    fn size_hint<const SIZE_BYTES: u8>(&self) -> Option<Sizes> {
        match *self {}
    }
}

/// Anything can be deserialized from `Never` formula,
/// because it is never exists in data, so deserialization of it never happens.
impl<'de> Deserialize<'de, Never> for Never {
    #[inline(always)]
    fn deserialize<D>(_deserializer: D) -> Result<Self, DeserializeError>
    where
        D: Deserializer<'de>,
    {
        unreachable!("Never formula should never be deserialized")
    }

    #[inline(always)]
    fn deserialize_in_place<D>(&mut self, _deserializer: D) -> Result<(), DeserializeError>
    where
        D: Deserializer<'de>,
    {
        unreachable!("Never formula should never be deserialized")
    }
}
