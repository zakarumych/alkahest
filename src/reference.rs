//!
//! This module provides formula for serializing unsized types through a reference.
//!

use core::{marker::PhantomData, mem::size_of};

use crate::{
    deserialize::{Deserialize, Deserializer, Error},
    formula::{BareFormula, Formula},
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

impl<F> Formula for Ref<F>
where
    F: BareFormula + ?Sized,
{
    const MAX_STACK_SIZE: Option<usize> = Some(size_of::<[FixedUsize; 2]>());
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
        let mut ser = ser.into();
        ser.write_ref::<F, T>(self)?;
        ser.finish()
    }

    #[inline(always)]
    fn fast_sizes(&self) -> Option<usize> {
        let size = self.fast_sizes()?;
        Some(size + size_of::<[FixedUsize; 2]>())
    }
}

impl<'de, F, T> Deserialize<'de, Ref<F>> for T
where
    F: BareFormula + ?Sized,
    T: Deserialize<'de, F> + ?Sized,
{
    #[inline(always)]
    fn deserialize(de: Deserializer<'de>) -> Result<T, Error>
    where
        T: Sized,
    {
        let de = de.deref()?;
        <T as Deserialize<F>>::deserialize(de)
    }

    #[inline(always)]
    fn deserialize_in_place(&mut self, de: Deserializer<'de>) -> Result<(), Error> {
        let de = de.deref()?;
        <T as Deserialize<F>>::deserialize_in_place(self, de)
    }
}
