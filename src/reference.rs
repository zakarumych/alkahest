//!
//! This module provides formula for serializing unsized types through a reference.
//!

use core::{marker::PhantomData, mem::size_of};

use crate::{
    deserialize::{Deserialize, Deserializer, Error},
    formula::{Formula, NonRefFormula},
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

impl<F: ?Sized> Formula for Ref<F>
where
    F: NonRefFormula,
{
    const MAX_SIZE: Option<usize> = Some(size_of::<[FixedUsize; 2]>());
}

impl<F, T> Serialize<Ref<F>> for T
where
    F: NonRefFormula + ?Sized,
    T: Serialize<F>,
{
    #[inline(always)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut ser = ser.into();
        ser.write_ref::<F, T>(self)?;
        ser.finish()
    }
}

impl<'de, F, T> Deserialize<'de, Ref<F>> for T
where
    F: NonRefFormula + ?Sized,
    T: Deserialize<'de, F> + ?Sized,
{
    #[inline(always)]
    fn deserialize(mut de: Deserializer<'de>) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let mut deref = de.deref()?;
        let value = deref.read_value::<F, T>()?;
        deref.finish()?;
        de.finish()?;
        Ok(value)
    }

    #[inline(always)]
    fn deserialize_in_place(&mut self, mut de: Deserializer<'de>) -> Result<(), Error> {
        let mut deref = de.deref()?;
        deref.read_in_place::<F, T>(self)?;
        deref.finish()?;
        de.finish()?;
        Ok(())
    }
}
