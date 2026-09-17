use alloc::string::String;

use crate::{
    cold_err,
    deserialize::{Deserialize, DeserializeError, Deserializer},
    element::Indirect,
    serialize::{Serialize, Serializer, Sizes},
    str::Str,
};

impl Serialize<Str> for String {
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
        Serialize::<Str>::size_hint::<SIZE_BYTES>(&self[..])
    }
}

impl<'de> Deserialize<'de, Str> for String {
    #[inline]
    fn deserialize<D>(mut deserializer: D) -> Result<Self, DeserializeError>
    where
        D: Deserializer<'de>,
    {
        let len = simple_try!(deserializer.read_usize());
        let bytes = simple_try!(deserializer.read_bytes(len));
        match core::str::from_utf8(bytes) {
            Ok(s) => Ok(String::from(s)),
            Err(error) => cold_err(DeserializeError::NonUtf8(error)),
        }
    }

    #[inline]
    fn deserialize_in_place<D>(&mut self, mut deserializer: D) -> Result<(), DeserializeError>
    where
        D: Deserializer<'de>,
    {
        let len = simple_try!(deserializer.read_usize());
        let bytes = simple_try!(deserializer.read_bytes(len));
        match core::str::from_utf8(bytes) {
            Ok(s) => {
                self.clear();
                self.push_str(s);
                Ok(())
            }
            Err(error) => cold_err(DeserializeError::NonUtf8(error)),
        }
    }
}

// String is commonly used in compound types,
// but `Str`
element_alias!(String as Indirect<Str>);
