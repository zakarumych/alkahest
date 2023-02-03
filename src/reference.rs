//!
//! This module provides formula for serializing unsized types through a reference.
//!

use core::{marker::PhantomData, mem::size_of};

use crate::{
    deserialize::{Deserialize, DeserializeError, Deserializer},
    formula::{Formula, NonRefFormula, UnsizedFormula},
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
pub struct Ref<F: ?Sized> {
    marker: PhantomData<fn(&F) -> &F>,
}

impl<F: ?Sized> UnsizedFormula for Ref<F> where F: NonRefFormula {}
impl<F: ?Sized> Formula for Ref<F>
where
    F: NonRefFormula,
{
    const SIZE: usize = <FixedUsize as Formula>::SIZE * 2;
}

impl<F, T> Serialize<Ref<F>> for T
where
    F: NonRefFormula + ?Sized,
    T: Serialize<F>,
{
    #[inline(always)]
    fn serialize(self, offset: usize, output: &mut [u8]) -> Result<(usize, usize), usize> {
        let mut ser = Serializer::new(offset, output);

        if let Err(size) = ser.serialize_unsized(self) {
            return Err(size + size_of::<[FixedUsize; 2]>());
        }

        let (address, size) = ser.flush();
        let address = FixedUsize::truncate_unchecked(address);
        let size = FixedUsize::truncate_unchecked(size);

        if let Err(size) = ser.serialize_self([address, size]) {
            return Err(size);
        }

        Ok(ser.finish())
    }

    #[inline(always)]
    fn size(self) -> usize {
        size_of::<[FixedUsize; 2]>() + <T as Serialize<F>>::size(self)
    }
}

impl<'de, F, T> Deserialize<'de, Ref<F>> for T
where
    F: NonRefFormula + ?Sized,
    T: Deserialize<'de, F> + ?Sized,
{
    fn deserialize(len: usize, input: &'de [u8]) -> Result<Self, DeserializeError>
    where
        Self: Sized,
    {
        let mut des = Deserializer::new(len, input)?;
        let [address, size] = des.deserialize_self::<[FixedUsize; 2]>()?;
        des.finish_expected();

        if usize::from(address) > input.len() {
            return Err(DeserializeError::WrongAddress);
        }

        let ref_input = &input[..usize::from(address)];

        let mut des = Deserializer::new(size.into(), ref_input)?;
        let value = des.deserialize::<F, T>(size.into())?;
        des.finish_expected();
        Ok(value)
    }

    fn deserialize_in_place(
        &mut self,
        len: usize,
        input: &'de [u8],
    ) -> Result<(), DeserializeError> {
        let mut des = Deserializer::new(len, input)?;
        let address = des.deserialize_self::<FixedUsize>()?;
        let size = des.deserialize_self::<FixedUsize>()?;
        des.finish_checked()?;

        if usize::from(address) > input.len() {
            return Err(DeserializeError::WrongAddress);
        }

        let ref_input = &input[..usize::from(address)];

        let mut des = Deserializer::new(size.into(), ref_input)?;
        des.deserialize_in_place::<F, T>(self, size.into())?;
        des.finish_expected();
        Ok(())
    }
}
