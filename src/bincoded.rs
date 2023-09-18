use std::{io::Cursor, marker::PhantomData, mem::size_of};

use crate::{
    buffer::Buffer,
    bytes::Bytes,
    deserialize::{Deserialize, DeserializeError, Deserializer},
    formula::{reference_size, Formula},
    serialize::{write_reference, Serialize, Sizes},
    size::{FixedUsize, FixedUsizeType},
};

/// A formula that can be used to serialize and deserialize data
/// using [`bincode`] crate.
///
/// Any type serializable with `serde` can be used with this formula.
/// If type is not serializable with `bincode` crate it will cause a panic.
/// Deserializing non-compatible type will cause deserialization error.
pub struct Bincode;

impl Formula for Bincode {
    const MAX_STACK_SIZE: Option<usize> = Some(reference_size::<Bytes>());
    const EXACT_SIZE: bool = true;
    const HEAPLESS: bool = false;
}

impl<T> Serialize<Bincode> for T
where
    T: serde::Serialize,
{
    #[inline]
    fn serialize<B>(self, sizes: &mut Sizes, mut buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        let options = bincode::config::DefaultOptions::new();

        let size = match bincode::Options::serialized_size(options, &self) {
            Ok(size) => size,
            Err(err) => panic!("Bincode serialization error: {}", err),
        };

        let Ok(size) = FixedUsizeType::try_from(size) else {
            panic!("Bincode serialization uses more that `FixedUsize::MAX` bytes");
        };

        let Ok(size) = FixedUsize::try_from(size) else {
            panic!("Bincode serialization uses more that `usize::MAX` bytes");
        };
        let size: usize = size.into();

        match buffer.reserve_heap(sizes.heap, sizes.stack, size) {
            Err(err) => return Err(err),
            Ok([]) => {} // Nothing to do.
            Ok(bytes) => {
                let mut cursor = Cursor::new(&mut bytes[sizes.heap..]);
                if let Err(err) = bincode::Options::serialize_into(options, &mut cursor, &self) {
                    panic!("Bincode serialization error: {}", err);
                };
                assert_eq!(cursor.position(), size as u64);
            }
        }

        sizes.heap += size;
        write_reference::<Bytes, B>(size, sizes.heap, sizes.heap, sizes.stack, buffer)?;
        sizes.stack += reference_size::<Bytes>();
        Ok(())
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        let options = bincode::config::DefaultOptions::new();

        let size = match bincode::Options::serialized_size(options, &self) {
            Ok(size) => size,
            Err(err) => panic!("Bincode serialization error: {}", err),
        };

        let Ok(size) = FixedUsizeType::try_from(size) else {
            panic!("Bincode serialization uses more that `FixedUsize::MAX` bytes");
        };

        let Ok(size) = FixedUsize::try_from(size) else {
            panic!("Bincode serialization uses more that `usize::MAX` bytes");
        };

        let size: usize = size.into();
        let mut sizes = Sizes::with_heap(size);
        sizes.add_stack(reference_size::<Bytes>());
        Some(sizes)
    }
}

impl<'de, T> Deserialize<'de, Bincode> for T
where
    T: serde::Deserialize<'de>,
{
    #[inline]
    fn deserialize(de: Deserializer<'de>) -> Result<Self, DeserializeError>
    where
        Self: Sized,
    {
        let de = de.deref::<Bytes>()?;

        let options = bincode::config::DefaultOptions::new();
        let mut de = bincode::de::Deserializer::from_slice(de.read_all_bytes(), options);

        match <T as serde::Deserialize<'de>>::deserialize(&mut de) {
            Ok(value) => Ok(value),
            Err(_err) => Err(DeserializeError::Incompatible),
        }
    }

    #[inline]
    fn deserialize_in_place(&mut self, de: Deserializer<'de>) -> Result<(), DeserializeError> {
        let de = de.deref::<Bytes>()?;

        let options = bincode::config::DefaultOptions::new();
        let mut de = bincode::de::Deserializer::from_slice(de.read_all_bytes(), options);

        match <T as serde::Deserialize<'de>>::deserialize_in_place(&mut de, self) {
            Ok(()) => Ok(()),
            Err(_err) => Err(DeserializeError::Incompatible),
        }
    }
}

/// A formula that can be used to serialize and deserialize data
/// using [`bincode`] crate.
///
/// Only one specified type can be used with this formula.
/// This helps avoid accidental deserialization of wrong type.
///
/// If type is not serializable with `bincode` crate it will cause a panic.
/// Deserializing non-compatible type will cause deserialization error.
pub struct Bincoded<T>(PhantomData<fn(&T) -> &T>);

impl<T> Formula for Bincoded<T> {
    const MAX_STACK_SIZE: Option<usize> = Some(size_of::<[FixedUsizeType; 2]>());
    const EXACT_SIZE: bool = true;
    const HEAPLESS: bool = false;
}

impl<T> Serialize<Bincoded<T>> for T
where
    T: serde::Serialize,
{
    #[inline(always)]
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        <T as Serialize<Bincode>>::serialize(self, sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        <T as Serialize<Bincode>>::size_hint(self)
    }
}

impl<'de, T> Deserialize<'de, Bincoded<T>> for T
where
    T: serde::Deserialize<'de>,
{
    #[inline(always)]
    fn deserialize(de: Deserializer<'de>) -> Result<Self, DeserializeError>
    where
        Self: Sized,
    {
        <T as Deserialize<'de, Bincode>>::deserialize(de)
    }

    #[inline(always)]
    fn deserialize_in_place(&mut self, de: Deserializer<'de>) -> Result<(), DeserializeError> {
        <T as Deserialize<'de, Bincode>>::deserialize_in_place(self, de)
    }
}
