//!
//! This module provides formula for serializing unsized types through a reference.
//!

use core::{marker::PhantomData, mem::size_of};

use crate::{
    deserialize::{Deserialize, DeserializeError, Deserializer},
    formula::{Formula, UnsizedFormula},
    serialize::{Serialize, Serializer},
    size::FixedUsize,
};

/// `Ref` is a formula wrapper.
/// It serializes the value in dynamic payload
/// and stores relative offset and the ref metadata.
/// Metadata is required for unsized types and is `()` for all sized types.
/// The `slice` type is unsized type that uses length metadata.
/// Structures allows last field to be of unsized type. In this case
/// metadata of the field inherited by the struct.
pub struct Ref<S: ?Sized> {
    marker: PhantomData<fn() -> S>,
}

impl<S: ?Sized> UnsizedFormula for Ref<S> where S: UnsizedFormula {}
impl<S: ?Sized> Formula for Ref<S>
where
    S: UnsizedFormula,
{
    const SIZE: usize = <FixedUsize as Formula>::SIZE * 2;
}

impl<S, T> Serialize<Ref<S>> for T
where
    S: UnsizedFormula + ?Sized,
    T: Serialize<S>,
{
    #[inline(always)]
    fn serialize(self, offset: usize, output: &mut [u8]) -> Result<(usize, usize), usize> {
        let mut ser = Serializer::new(offset, output);

        if let Err(size) = ser.serialize_value(self) {
            return Err(size + size_of::<[FixedUsize; 2]>());
        }

        let (address, size) = ser.flush();
        let address = FixedUsize::truncated(address);
        let size = FixedUsize::truncated(size);

        if let Err(size) = ser.serialize_self([address, size]) {
            return Err(size);
        }

        Ok(ser.finish())
    }

    #[inline(always)]
    fn size(self) -> usize {
        size_of::<[FixedUsize; 2]>() + <T as Serialize<S>>::size(self)
    }
}

impl<'a, S, T> Deserialize<'a, Ref<S>> for T
where
    S: UnsizedFormula + ?Sized,
    T: Deserialize<'a, S>,
{
    fn deserialize(len: usize, input: &'a [u8]) -> Result<Self, DeserializeError> {
        let mut des = Deserializer::new(len, input);
        let [address, size] = des.deserialize_self::<[FixedUsize; 2]>()?;
        des.finish_expected();

        let ref_input = &input[..usize::from(address)];

        let mut des = Deserializer::new(size.into(), ref_input);
        let value = des.deserialize::<S, T>(size.into())?;
        des.finish_expected();
        Ok(value)
    }

    fn deserialize_in_place(
        &mut self,
        len: usize,
        input: &'a [u8],
    ) -> Result<(), DeserializeError> {
        if len != size_of::<[FixedUsize; 2]>() {
            return Err(DeserializeError::WrongLength);
        }

        let mut des = Deserializer::new(len, input);
        let address = des.deserialize_self::<FixedUsize>()?;
        let size = des.deserialize_self::<FixedUsize>()?;
        des.finish_expected();

        let ref_input = &input[..usize::from(address)];

        let mut des = Deserializer::new(size.into(), ref_input);
        des.deserialize_in_place::<S, T>(self, size.into())?;
        des.finish_expected();
        Ok(())
    }
}
