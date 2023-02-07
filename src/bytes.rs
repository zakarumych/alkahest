use crate::{
    deserialize::{Deserialize, Deserializer, Error},
    formula::NonRefFormula,
    serialize::{Serialize, Serializer},
};

/// A formula for a raw byte slices.
/// Serializable from anything that implements `AsRef<[u8]>`.
pub struct Bytes;

impl NonRefFormula for Bytes {
    const MAX_SIZE: Option<usize> = None;
}

impl Serialize<Bytes> for &[u8] {
    #[inline(always)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut ser = ser.into();
        ser.write_bytes(self.as_ref())?;
        ser.finish()
    }
}

impl<'de, 'fe: 'de> Deserialize<'fe, Bytes> for &'de [u8] {
    #[inline(always)]
    fn deserialize(de: Deserializer<'fe>) -> Result<Self, Error> {
        Ok(de.read_all_bytes())
    }

    #[inline(always)]
    fn deserialize_in_place(&mut self, de: Deserializer<'fe>) -> Result<(), Error> {
        *self = de.read_all_bytes();
        Ok(())
    }
}
