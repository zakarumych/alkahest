use core::mem::size_of;

use alloc::{borrow::ToOwned, string::String};

use crate::{
    deserialize::{Deserialize, Deserializer, Error},
    formula::Formula,
    reference::Ref,
    serialize::{Serialize, Serializer},
    FixedUsize,
};

impl Formula for String {
    const MAX_STACK_SIZE: Option<usize> = <Ref<str> as Formula>::MAX_STACK_SIZE;
    const EXACT_SIZE: bool = <Ref<str> as Formula>::EXACT_SIZE;
    const HEAPLESS: bool = <Ref<str> as Formula>::HEAPLESS;
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
        ser.into().write_ref::<str, T>(self)
    }

    #[inline(always)]
    fn fast_sizes(&self) -> Option<usize> {
        let size = self.fast_sizes()?;
        Some(size + size_of::<[FixedUsize; 2]>())
    }
}

impl<'de, T> Deserialize<'de, String> for T
where
    T: Deserialize<'de, str>,
{
    #[inline(always)]
    fn deserialize(de: Deserializer<'de>) -> Result<T, Error> {
        let de = de.deref()?;
        <T as Deserialize<str>>::deserialize(de)
    }

    #[inline(always)]
    fn deserialize_in_place(&mut self, de: Deserializer<'de>) -> Result<(), Error> {
        let de = de.deref()?;
        <T as Deserialize<str>>::deserialize_in_place(self, de)
    }
}

impl Serialize<str> for String {
    #[inline(always)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut ser = ser.into();
        ser.write_bytes(self.as_bytes())?;
        ser.finish()
    }

    #[inline(always)]
    fn fast_sizes(&self) -> Option<usize> {
        Some(self.len())
    }
}

impl Serialize<str> for &String {
    #[inline(always)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut ser = ser.into();
        ser.write_bytes(self.as_bytes())?;
        ser.finish()
    }

    #[inline(always)]
    fn fast_sizes(&self) -> Option<usize> {
        Some(self.len())
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
