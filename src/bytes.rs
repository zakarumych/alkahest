use crate::{
    buffer::Buffer,
    deserialize::{Deserialize, DeserializeError, Deserializer},
    formula::{BareFormula, Formula},
    serialize::{write_bytes, Serialize, Sizes},
};

/// A formula for a raw byte slices.
/// Serializable from anything that implements `AsRef<[u8]>`.
pub struct Bytes;

impl Formula for Bytes {
    const MAX_STACK_SIZE: Option<usize> = None;
    const EXACT_SIZE: bool = false;
    const HEAPLESS: bool = true;
}

impl BareFormula for Bytes {}

impl Serialize<Bytes> for &[u8] {
    #[inline(always)]
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        write_bytes(self, sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        Some(Sizes::with_stack(self.len()))
    }
}

impl<'de, 'fe: 'de> Deserialize<'fe, Bytes> for &'de [u8] {
    #[inline(always)]
    fn deserialize(de: Deserializer<'fe>) -> Result<Self, DeserializeError> {
        Ok(de.read_all_bytes())
    }

    #[inline(always)]
    fn deserialize_in_place(&mut self, de: Deserializer<'fe>) -> Result<(), DeserializeError> {
        *self = de.read_all_bytes();
        Ok(())
    }
}
