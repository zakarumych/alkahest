//!
//! This module provides formula for serializing unsized types through a reference.
//!

use core::marker::PhantomData;

use crate::{
    deserialize::{Deserialize, DeserializeError, Deserializer},
    formula::{BareFormula, Formula},
    serialize::{field_size_hint, reference_size, Serialize, Serializer},
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

impl<F> Formula for Ref<F>
where
    F: BareFormula + ?Sized,
{
    const MAX_STACK_SIZE: Option<usize> = Some(reference_size::<F>());
    const EXACT_SIZE: bool = true;
    const HEAPLESS: bool = false;
}

impl<F, T> Serialize<Ref<F>> for T
where
    F: BareFormula + ?Sized,
    T: Serialize<F>,
{
    #[inline(always)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        ser.into().write_ref::<F, T>(self)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<(usize, usize)> {
        let (heap, stack) = field_size_hint::<F>(self, true)?;
        Some((heap + stack, reference_size::<F>()))
    }
}

impl<'de, F, T> Deserialize<'de, Ref<F>> for T
where
    F: BareFormula + ?Sized,
    T: Deserialize<'de, F> + ?Sized,
{
    #[inline(always)]
    fn deserialize(de: Deserializer<'de>) -> Result<T, DeserializeError>
    where
        T: Sized,
    {
        let de = de.deref::<F>()?;
        <T as Deserialize<F>>::deserialize(de)
    }

    #[inline(always)]
    fn deserialize_in_place(&mut self, de: Deserializer<'de>) -> Result<(), DeserializeError> {
        let de = de.deref::<F>()?;
        <T as Deserialize<F>>::deserialize_in_place(self, de)
    }
}
