use core::marker::PhantomData;

use crate::{
    buffer::Buffer,
    deserialize::{Deserialize, DeserializeError, Deserializer},
    formula::{BareFormulaType, FormulaType},
    serialize::{Serialize, Sizes},
    SerializeRef,
};

#[cfg(feature = "evolution")]
use crate::evolution::Descriptor;

/// Formula type that mirrors specified formula `F`.
/// It can be used to turn unsized field type into sized one,
/// keeping the same formula.
///
/// # Example
///
/// ```compile_fail
/// # use alkahest::*;
/// type MyFormula = [str]; // Slice element type must be sized. `str` is unsized.
///
/// let mut buffer = [0u8; 22];
/// serialize::<MyFormula, _>(["qwe", "rty"], &mut buffer).unwrap();
/// ```
///
/// // Wrap usized type into `As`
///
/// ```
/// # use alkahest::*;
/// type MyFormula = [As<str>]; // `As` is always size.
///
/// # #[cfg(feature = "fixed8")]
/// # let mut buffer = [0u8; 10];
///
/// # #[cfg(feature = "fixed16")]
/// # let mut buffer = [0u8; 14];
///
/// # #[cfg(feature = "fixed32")]
/// let mut buffer = [0u8; 22];
///
/// # #[cfg(feature = "fixed64")]
/// # let mut buffer = [0u8; 38];
/// serialize::<MyFormula, _>(["qwe", "rty"], &mut buffer).unwrap();
/// ```
pub struct As<F: ?Sized> {
    marker: PhantomData<fn(&F) -> &F>,
}

impl<F> FormulaType for As<F>
where
    F: BareFormulaType + ?Sized,
{
    const MAX_STACK_SIZE: Option<usize> = F::MAX_STACK_SIZE;
    const EXACT_SIZE: bool = F::EXACT_SIZE;
    const HEAPLESS: bool = F::HEAPLESS;

    #[cfg(feature = "evolution")]
    fn descriptor(builder: crate::evolution::DescriptorBuilder) {
        F::descriptor(builder)
    }
}

impl<F, T> Serialize<As<F>> for T
where
    F: BareFormulaType + ?Sized,
    T: Serialize<F>,
{
    #[inline(always)]
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        Self: Sized,
        B: Buffer,
    {
        <T as Serialize<F>>::serialize(self, sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        <T as Serialize<F>>::size_hint(self)
    }
}

impl<F, T> SerializeRef<As<F>> for T
where
    F: BareFormulaType + ?Sized,
    T: ?Sized,
    for<'a> &'a T: Serialize<F>,
{
    #[inline(always)]
    fn serialize<B>(&self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        <&T as Serialize<F>>::serialize(self, sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        <&T as Serialize<F>>::size_hint(&self)
    }
}

impl<'de, F, T> Deserialize<'de, As<F>> for T
where
    F: BareFormulaType + ?Sized,
    T: Deserialize<'de, F>,
{
    #[inline(always)]
    fn deserialize(deserializer: Deserializer<'de>) -> Result<Self, DeserializeError>
    where
        Self: Sized,
    {
        <T as Deserialize<'de, F>>::deserialize(deserializer)
    }

    #[inline(always)]
    fn deserialize_in_place(
        &mut self,
        deserializer: Deserializer<'de>,
    ) -> Result<(), DeserializeError> {
        <T as Deserialize<'de, F>>::deserialize_in_place(self, deserializer)
    }

    #[cfg(feature = "evolution")]
    #[inline(always)]
    fn deserialize_with_descriptor(
        descriptor: &Descriptor,
        formula: u32,
        deserializer: Deserializer<'de>,
    ) -> Result<Self, DeserializeError>
    where
        Self: Sized,
    {
        <T as Deserialize<'de, F>>::deserialize_with_descriptor(descriptor, formula, deserializer)
    }

    #[cfg(feature = "evolution")]
    #[inline(always)]
    fn deserialize_in_place_with_descriptor(
        &mut self,
        descriptor: &Descriptor,
        formula: u32,
        deserializer: Deserializer<'de>,
    ) -> Result<(), DeserializeError> {
        <T as Deserialize<'de, F>>::deserialize_in_place_with_descriptor(
            self,
            descriptor,
            formula,
            deserializer,
        )
    }
}
