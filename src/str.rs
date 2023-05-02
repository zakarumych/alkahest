use crate::{
    buffer::Buffer,
    deserialize::{Deserialize, DeserializeError, Deserializer},
    formula::{BareFormula, Formula},
    serialize::{write_bytes, SerializeRef, Sizes},
};

impl Formula for str {
    const MAX_STACK_SIZE: Option<usize> = None;
    const EXACT_SIZE: bool = false;
    const HEAPLESS: bool = true;
}

impl BareFormula for str {}

impl SerializeRef<str> for str {
    #[inline(always)]
    fn serialize<B>(&self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        write_bytes(self.as_bytes(), sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        Some(Sizes::with_stack(self.len()))
    }
}

impl<'de, 'fe: 'de> Deserialize<'fe, str> for &'de str {
    #[inline(always)]
    fn deserialize(deserializer: Deserializer<'fe>) -> Result<Self, DeserializeError>
    where
        Self: Sized,
    {
        let bytes = deserializer.read_all_bytes();
        match core::str::from_utf8(bytes) {
            Ok(s) => Ok(s),
            Err(error) => Err(DeserializeError::NonUtf8(error)),
        }
    }

    #[inline(always)]
    fn deserialize_in_place(
        &mut self,
        deserializer: Deserializer<'fe>,
    ) -> Result<(), DeserializeError> {
        let bytes = deserializer.read_all_bytes();
        match core::str::from_utf8(bytes) {
            Ok(s) => {
                *self = s;
                Ok(())
            }
            Err(error) => Err(DeserializeError::NonUtf8(error)),
        }
    }
}
