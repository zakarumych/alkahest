//!
//! This module provides schema for serializing unsized types through a reference.
//!

use core::{marker::PhantomData, mem::size_of};

use crate::{
    deserialize::{Deserialize, DeserializeError, Deserializer},
    schema::Schema,
    serialize::{Serialize, Serializer},
    size::FixedUsize,
};

/// `Ref` is a schema wrapper.
/// It serializes the value in dynamic payload
/// and stores relative offset and the ref metadata.
/// Metadata is required for unsized types and is `()` for all sized types.
/// The `slice` type is unsized type that uses length metadata.
/// Structures allows last field to be of unsized type. In this case
/// metadata of the field inherited by the struct.
pub struct Ref<S: ?Sized> {
    marker: PhantomData<fn() -> S>,
}

impl<S: ?Sized> Schema for Ref<S> where S: Schema {}

impl<S, T> Serialize<Ref<S>> for T
where
    S: Schema + ?Sized,
    T: Serialize<S>,
{
    #[inline(always)]
    fn serialize(self, offset: usize, output: &mut [u8]) -> Result<(usize, usize), usize> {
        let mut ser = Serializer::new(offset, output);
        if let Err(size) = ser.put(self) {
            return Err(size + size_of::<FixedUsize>());
        }

        ser.flush();

        let address = FixedUsize::truncated(ser.offset());
        if let Err(size) = ser.put::<FixedUsize, _>(address) {
            return Err(size + ser.written());
        }

        Ok(ser.finish())
    }
}

impl<'a, S, T> Deserialize<'a, Ref<S>> for T
where
    S: Schema + ?Sized,
    T: Deserialize<'a, S>,
{
    fn deserialize(input: &'a [u8]) -> Result<(Self, usize), DeserializeError> {
        let mut des = Deserializer::new(input);
        let address = des.deserialize::<FixedUsize, FixedUsize>()?;

        let (value, _) = T::deserialize(&input[..address.into()])?;
        Ok((value, des.end()))
    }

    fn deserialize_in_place(&mut self, input: &'a [u8]) -> Result<usize, DeserializeError> {
        let mut des = Deserializer::new(input);
        let address = des.deserialize::<FixedUsize, FixedUsize>()?;

        T::deserialize_in_place(self, &input[..address.into()])?;
        Ok(des.end())
    }
}
