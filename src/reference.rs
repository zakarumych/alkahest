//!
//! This module provides formula for serializing unsized types through a reference.
//!

use core::{marker::PhantomData, mem::size_of};

use crate::{
    deserialize::{Deserializer, Error, NonRefDeserialize},
    formula::{Formula, NonRefFormula},
    serialize::{SerializeOwned, Serializer},
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

impl<F: ?Sized> Formula for Ref<F>
where
    F: NonRefFormula,
{
    const MAX_SIZE: Option<usize> = Some(size_of::<[FixedUsize; 2]>());

    type NonRef = F;

    #[inline(always)]
    fn serialize<T, S>(value: T, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        T: SerializeOwned<F>,
        S: Serializer,
    {
        let mut ser = ser.into();
        ser.write_ref::<F, T>(value)?;
        ser.finish()
    }

    #[inline(always)]
    fn deserialize<'de, T>(de: Deserializer<'de>) -> Result<T, Error>
    where
        T: NonRefDeserialize<'de, F>,
    {
        let mut de = de.deref()?;
        let value = de.read_value::<F, T>()?;
        de.finish()?;
        Ok(value)
    }

    #[inline(always)]
    fn deserialize_in_place<'de, T>(place: &mut T, de: Deserializer<'de>) -> Result<(), Error>
    where
        T: NonRefDeserialize<'de, F> + ?Sized,
    {
        let mut de = de.deref()?;
        de.read_in_place::<F, T>(place)?;
        de.finish()?;
        Ok(())
    }
}
