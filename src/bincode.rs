use core::marker::PhantomData;

use crate::{
    bytes::Bytes,
    deserialize::{Deserialize, Deserializer},
    formula::Formula,
    serialize::{Serialize, Serializer},
};

/// A formula that can be used to serialize and deserialize data
/// using [`bincode`] crate.
pub struct Bincode;

impl Formula for Bincode {
    const MAX_STACK_SIZE: Option<usize> = None;
    const EXACT_SIZE: bool = false;
    const HEAPLESS: bool = false;
}

impl<T> Serialize<Bincode> for T
where
    T: serde::Serialize,
{
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let options = bincode::config::DefaultOptions::new();

        match bincode::Options::serialize(options, &self) {
            Ok(bytes) => <_ as Serialize<Bytes>>::serialize(bytes, ser),
            Err(err) => panic!("Bincode serialization error: {}", err),
        }
    }

    fn fast_sizes(&self) -> Option<usize> {
        None
    }
}

impl<'de, T> Deserialize<'de, Bincode> for T
where
    T: serde::Deserialize<'de>,
{
    fn deserialize(de: Deserializer<'de>) -> Result<Self, crate::Error>
    where
        Self: Sized,
    {
        let options = bincode::config::DefaultOptions::new();
        let mut de = bincode::de::Deserializer::from_slice(de.read_all_bytes(), options);

        match <T as serde::Deserialize<'de>>::deserialize(&mut de) {
            Ok(value) => Ok(value),
            Err(err) => panic!("Bincode deserialization error: {}", err),
        }
    }

    fn deserialize_in_place(&mut self, de: Deserializer<'de>) -> Result<(), crate::Error> {
        let options = bincode::config::DefaultOptions::new();
        let mut de = bincode::de::Deserializer::from_slice(de.read_all_bytes(), options);

        match <T as serde::Deserialize<'de>>::deserialize_in_place(&mut de, self) {
            Ok(()) => Ok(()),
            Err(err) => panic!("Bincode deserialization error: {}", err),
        }
    }
}

/// A formula that can be used to serialize and deserialize data
/// using [`bincode`] crate.
pub struct Bincoded<T>(PhantomData<fn(&T) -> &T>);

impl<T> Formula for Bincoded<T> {
    const MAX_STACK_SIZE: Option<usize> = None;
    const EXACT_SIZE: bool = false;
    const HEAPLESS: bool = false;
}

impl<T> Serialize<Bincoded<T>> for T
where
    T: serde::Serialize,
{
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let options = bincode::config::DefaultOptions::new();

        match bincode::Options::serialize(options, &self) {
            Ok(bytes) => <_ as Serialize<Bytes>>::serialize(bytes, ser),
            Err(err) => panic!("Bincode serialization error: {}", err),
        }
    }

    fn fast_sizes(&self) -> Option<usize> {
        None
    }
}

impl<'de, T> Deserialize<'de, Bincoded<T>> for T
where
    T: serde::Deserialize<'de>,
{
    fn deserialize(de: Deserializer<'de>) -> Result<Self, crate::Error>
    where
        Self: Sized,
    {
        let options = bincode::config::DefaultOptions::new();
        let mut de = bincode::de::Deserializer::from_slice(de.read_all_bytes(), options);

        match <T as serde::Deserialize<'de>>::deserialize(&mut de) {
            Ok(value) => Ok(value),
            Err(err) => panic!("Bincode deserialization error: {}", err),
        }
    }

    fn deserialize_in_place(&mut self, de: Deserializer<'de>) -> Result<(), crate::Error> {
        let options = bincode::config::DefaultOptions::new();
        let mut de = bincode::de::Deserializer::from_slice(de.read_all_bytes(), options);

        match <T as serde::Deserialize<'de>>::deserialize_in_place(&mut de, self) {
            Ok(()) => Ok(()),
            Err(err) => panic!("Bincode deserialization error: {}", err),
        }
    }
}
