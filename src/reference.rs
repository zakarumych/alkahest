//!
//! This module provides formula for serializing unsized types through a reference.
//!

use core::marker::PhantomData;

use crate::{
    buffer::Buffer,
    deserialize::{Deserialize, DeserializeError, Deserializer},
    formula::{reference_size, BareFormulaType, FormulaType},
    serialize::{field_size_hint, write_ref, write_reference, Serialize, Sizes},
    SerializeRef,
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

impl<F> FormulaType for Ref<F>
where
    F: BareFormulaType + ?Sized,
{
    const MAX_STACK_SIZE: Option<usize> = Some(reference_size::<F>());
    const EXACT_SIZE: bool = true;
    const HEAPLESS: bool = matches!(F::MAX_STACK_SIZE, Some(0));

    #[cfg(feature = "evolution")]
    fn descriptor(builder: crate::evolution::DescriptorBuilder) {
        builder.reference::<F>();
    }
}

impl<F, T> Serialize<Ref<F>> for T
where
    F: BareFormulaType + ?Sized,
    T: Serialize<F>,
{
    #[inline(always)]
    fn serialize<B>(self, sizes: &mut Sizes, mut buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        let size = write_ref::<F, T, _>(self, sizes, buffer.reborrow())?;
        write_reference::<F, B>(size, sizes.heap, sizes.heap, sizes.stack, buffer)?;
        sizes.stack += reference_size::<F>();
        Ok(())
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        let mut sizes = field_size_hint::<F>(self, true)?;
        sizes.to_heap(0);
        sizes.add_stack(reference_size::<F>());
        Some(sizes)
    }
}

impl<F, T> SerializeRef<Ref<F>> for T
where
    F: BareFormulaType + ?Sized,
    T: ?Sized,
    for<'a> &'a T: Serialize<F>,
{
    #[inline(always)]
    fn serialize<B>(&self, sizes: &mut Sizes, mut buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        let size = write_ref::<F, &T, _>(self, sizes, buffer.reborrow())?;
        write_reference::<F, B>(size, sizes.heap, sizes.heap, sizes.stack, buffer)?;
        sizes.stack += reference_size::<F>();
        Ok(())
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        let mut sizes = field_size_hint::<F>(&self, true)?;
        sizes.to_heap(0);
        sizes.add_stack(reference_size::<F>());
        Some(sizes)
    }
}

impl<'de, F, T> Deserialize<'de, Ref<F>> for T
where
    F: BareFormulaType + ?Sized,
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

    #[inline(always)]
    fn deserialize_with_descriptor(
        desc: &crate::evolution::Descriptor,
        formula: u32,
        de: Deserializer<'de>,
    ) -> Result<Self, DeserializeError>
    where
        Self: Sized,
    {
        match *desc.get(formula) {
            crate::evolution::Flavor::Ref(None) => {
                let de = de.deref::<F>()?;
                <T as Deserialize<F>>::deserialize(de)
            }
            crate::evolution::Flavor::Ref(Some(inner)) => {
                let de = de.deref::<F>()?;
                <T as Deserialize<F>>::deserialize_with_descriptor(desc, inner, de)
            }
            // Data is not a reference. Passthrough.
            _ => <T as Deserialize<F>>::deserialize_with_descriptor(desc, formula, de),
        }
    }

    #[inline(always)]
    fn deserialize_in_place_with_descriptor(
        &mut self,
        desc: &crate::evolution::Descriptor,
        formula: u32,
        de: Deserializer<'de>,
    ) -> Result<(), DeserializeError> {
        match *desc.get(formula) {
            crate::evolution::Flavor::Ref(None) => {
                let de = de.deref::<F>()?;
                <T as Deserialize<F>>::deserialize_in_place(self, de)
            }
            crate::evolution::Flavor::Ref(Some(inner)) => {
                let de = de.deref::<F>()?;
                <T as Deserialize<F>>::deserialize_in_place_with_descriptor(self, desc, inner, de)
            }
            // Data is not a reference. Passthrough.
            _ => {
                <T as Deserialize<F>>::deserialize_in_place_with_descriptor(self, desc, formula, de)
            }
        }
    }
}
