use core::marker::PhantomData;

use crate::{
    deserialize::{Deserialize, DeserializeError, Deserializer},
    formula::{BareFormula, Formula},
    serialize::{Serialize, Serializer},
};

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

impl<F> Formula for As<F>
where
    F: BareFormula + ?Sized,
{
    const MAX_STACK_SIZE: Option<usize> = F::MAX_STACK_SIZE;
    const EXACT_SIZE: bool = F::EXACT_SIZE;
    const HEAPLESS: bool = F::HEAPLESS;
}

impl<F, T> Serialize<As<F>> for T
where
    F: BareFormula + ?Sized,
    T: Serialize<F>,
{
    #[inline(always)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        Self: Sized,
        S: Serializer,
    {
        <T as Serialize<F>>::serialize(self, ser)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<usize> {
        <T as Serialize<F>>::size_hint(self)
    }
}

impl<'de, F, T> Deserialize<'de, As<F>> for T
where
    F: BareFormula + ?Sized,
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
}
