use alloc::{borrow::ToOwned, string::String};

use crate::{
    deserialize::{Deserialize, Deserializer, Error},
    formula::Formula,
    reference::Ref,
    serialize::{Serialize, Serializer},
};

impl Formula for String {
    const MAX_SIZE: Option<usize> = <Ref<str> as Formula>::MAX_SIZE;

    type NonRef = str;
}

impl<T> Serialize<String> for T
where
    T: Serialize<str>,
{
    #[inline(always)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        T: Serialize<str>,
        S: Serializer,
    {
        <T as Serialize<Ref<str>>>::serialize(self, ser)
    }
}

impl<'de, T> Deserialize<'de, String> for T
where
    T: Deserialize<'de, str>,
{
    #[inline(always)]
    fn deserialize(de: Deserializer<'de>) -> Result<T, Error> {
        <T as Deserialize<'de, Ref<str>>>::deserialize(de)
    }

    #[inline(always)]
    fn deserialize_in_place(&mut self, de: Deserializer<'de>) -> Result<(), Error> {
        <T as Deserialize<'de, Ref<str>>>::deserialize_in_place(self, de)
    }
}

impl Serialize<str> for String {
    #[inline(always)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        <&str as Serialize<str>>::serialize(&self, ser)
    }
}

impl Serialize<str> for &String {
    #[inline(always)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        <&str as Serialize<str>>::serialize(self, ser)
    }
}

impl<'de> Deserialize<'de, str> for String {
    #[inline(always)]
    fn deserialize(deserializer: Deserializer<'de>) -> Result<Self, Error> {
        let string = <&str as Deserialize<'de, str>>::deserialize(deserializer)?;
        Ok(string.to_owned())
    }

    #[inline(always)]
    fn deserialize_in_place(&mut self, deserializer: Deserializer<'de>) -> Result<(), Error> {
        let string = <&str as Deserialize<'de, str>>::deserialize(deserializer)?;
        self.push_str(string);
        Ok(())
    }
}
