use core::marker::PhantomData;

use crate::{
    deserialize::{Deserialize, Deserializer, Error},
    formula::Formula,
    private::BareFormula,
    serialize::{Serialize, Serializer},
};

/// Formula type that mirrors specified formula `F`.
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
    fn fast_sizes(&self) -> Option<usize> {
        <T as Serialize<F>>::fast_sizes(self)
    }
}

impl<'de, F, T> Deserialize<'de, As<F>> for T
where
    F: BareFormula + ?Sized,
    T: Deserialize<'de, F>,
{
    #[inline(always)]
    fn deserialize(deserializer: Deserializer<'de>) -> Result<Self, Error>
    where
        Self: Sized,
    {
        <T as Deserialize<'de, F>>::deserialize(deserializer)
    }

    #[inline(always)]
    fn deserialize_in_place(&mut self, deserializer: Deserializer<'de>) -> Result<(), Error> {
        <T as Deserialize<'de, F>>::deserialize_in_place(self, deserializer)
    }
}
