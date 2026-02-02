use std::str;

use crate::{
    deserialize::{Deserialize, DeserializeError, Deserializer},
    formula::{ExactSize, Formula, UnboundedSize},
    serialize::{Serialize, Serializer, Sizes},
};

impl Formula for str {
    type StackSize<const SIZE_BYTES: u8> = UnboundedSize;
    type HeapSize<const SIZE_BYTES: u8> = ExactSize<0>;
    const INHABITED: bool = true;
}

impl Serialize<str> for str {
    #[inline]
    fn serialize<S>(&self, mut serializer: S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        serializer.write_usize(self.len())?;
        serializer.write_bytes(self.as_bytes())
    }

    #[inline]
    fn size_hint<const SIZE_BYTES: u8>(&self) -> Option<Sizes> {
        let mut sizes = Sizes::with_stack(usize::from(SIZE_BYTES));
        sizes.add_stack(self.len());
        Some(sizes)
    }
}

impl<'de, 'fe: 'de> Deserialize<'fe, str> for &'de str {
    #[inline]
    fn deserialize<D>(mut deserializer: D) -> Result<Self, DeserializeError>
    where
        D: Deserializer<'fe>,
    {
        let len = deserializer.read_usize()?;
        let bytes = deserializer.read_bytes(len)?;
        match core::str::from_utf8(bytes) {
            Ok(s) => Ok(s),
            Err(error) => Err(DeserializeError::NonUtf8(error)),
        }
    }

    #[inline]
    fn deserialize_in_place<D>(&mut self, mut deserializer: D) -> Result<(), DeserializeError>
    where
        D: Deserializer<'fe>,
    {
        let len = deserializer.read_usize()?;
        let bytes = deserializer.read_bytes(len)?;
        match core::str::from_utf8(bytes) {
            Ok(s) => {
                *self = s;
                Ok(())
            }
            Err(error) => Err(DeserializeError::NonUtf8(error)),
        }
    }
}
