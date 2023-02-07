use crate::{
    deserialize::{Deserialize, Deserializer, Error},
    formula::Formula,
    private::NonRefFormula,
    serialize::{Serialize, Serializer},
};

impl NonRefFormula for str {
    const MAX_SIZE: Option<usize> = <[u8] as Formula>::MAX_SIZE;
}

impl Serialize<str> for &str {
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut ser = ser.into();
        ser.write_bytes(self.as_bytes())?;
        ser.finish()
    }
}

impl<'de, 'fe: 'de> Deserialize<'fe, str> for &'de str {
    fn deserialize(deserializer: Deserializer<'fe>) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let bytes = deserializer.read_all_bytes();
        match core::str::from_utf8(bytes) {
            Ok(s) => Ok(s),
            Err(err) => Err(Error::NonUtf8(err)),
        }
    }

    fn deserialize_in_place(&mut self, deserializer: Deserializer<'fe>) -> Result<(), Error> {
        let bytes = deserializer.read_all_bytes();
        match core::str::from_utf8(bytes) {
            Ok(s) => {
                *self = s;
                Ok(())
            }
            Err(err) => Err(Error::NonUtf8(err)),
        }
    }
}
