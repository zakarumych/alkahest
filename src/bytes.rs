use crate::{
    deserialize::{Deserialize, Deserializer, Error},
    formula::NonRefFormula,
    serialize::{SerializeOwned, Serializer},
};

/// A formula for a raw byte slices.
/// Serializable from anything that implements `AsRef<[u8]>`.
pub struct Bytes;

impl NonRefFormula for Bytes {
    const MAX_SIZE: Option<usize> = None;
}

impl SerializeOwned<Bytes> for &[u8] {
    #[cfg_attr(feature = "inline-more", inline(always))]
    fn serialize_owned<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut ser = ser.into();
        ser.write_bytes(self.as_ref())?;
        ser.finish()
    }
}

impl<'de> Deserialize<'de, Bytes> for &'de [u8] {
    #[cfg_attr(feature = "inline-more", inline(always))]
    fn deserialize(de: Deserializer<'de>) -> Result<Self, Error> {
        Ok(de.read_all_bytes())
    }

    #[cfg_attr(feature = "inline-more", inline(always))]
    fn deserialize_in_place(&mut self, de: Deserializer<'de>) -> Result<(), Error> {
        *self = de.read_all_bytes();
        Ok(())
    }
}
