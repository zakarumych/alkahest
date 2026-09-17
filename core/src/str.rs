use crate::{
    cold_err,
    deserialize::{Deserialize, DeserializeError, Deserializer},
    formula::{ExactSize, Formula, UnboundedSize},
    serialize::{Serialize, Serializer, Sizes},
};

/// Formula representing a string slice.
pub struct Str;

impl Formula for Str {
    type StackSize<const SIZE_BYTES: usize> = UnboundedSize;
    type HeapSize<const SIZE_BYTES: usize> = ExactSize<0>;
    const INHABITED: bool = true;
}

impl Serialize<Str> for str {
    #[inline(always)]
    fn serialize<S>(&self, mut serializer: S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        simple_try!(serializer.write_usize(self.len()));
        serializer.write_bytes(self.as_bytes())
    }

    #[inline(always)]
    fn size_hint<const SIZE_BYTES: usize>(&self) -> Option<Sizes> {
        let mut sizes = Sizes::with_stack(SIZE_BYTES);
        sizes.add_stack(self.len());
        Some(sizes)
    }
}

impl<'de, 'fe: 'de> Deserialize<'fe, Str> for &'de str {
    #[inline]
    fn deserialize<D>(mut deserializer: D) -> Result<Self, DeserializeError>
    where
        D: Deserializer<'fe>,
    {
        let len = simple_try!(deserializer.read_usize());
        let bytes = simple_try!(deserializer.read_bytes(len));
        match core::str::from_utf8(bytes) {
            Ok(s) => Ok(s),
            Err(error) => cold_err(DeserializeError::NonUtf8(error)),
        }
    }

    #[inline]
    fn deserialize_in_place<D>(&mut self, mut deserializer: D) -> Result<(), DeserializeError>
    where
        D: Deserializer<'fe>,
    {
        let len = simple_try!(deserializer.read_usize());
        let bytes = simple_try!(deserializer.read_bytes(len));
        match core::str::from_utf8(bytes) {
            Ok(s) => {
                *self = s;
                Ok(())
            }
            Err(error) => cold_err(DeserializeError::NonUtf8(error)),
        }
    }
}
