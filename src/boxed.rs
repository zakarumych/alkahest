use alloc::{boxed::Box, rc::Rc, sync::Arc};

use crate::{
    buffer::Buffer,
    deserialize::{Deserialize, DeserializeError, Deserializer},
    formula::BareFormula,
    serialize::{Serialize, Sizes},
    SerializeRef,
};

impl<T, F> Serialize<F> for Box<T>
where
    F: BareFormula,
    T: Serialize<F>,
{
    #[inline(always)]
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        <T as Serialize<F>>::serialize(*self, sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        <T as Serialize<F>>::size_hint(self)
    }
}

impl<T, F> SerializeRef<F> for Box<T>
where
    F: BareFormula,
    T: SerializeRef<F>,
{
    #[inline(always)]
    fn serialize<B>(&self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        <T as SerializeRef<F>>::serialize(self.as_ref(), sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        <T as SerializeRef<F>>::size_hint(self.as_ref())
    }
}

impl<'de, T, F> Deserialize<'de, F> for Box<T>
where
    F: BareFormula,
    T: Deserialize<'de, F>,
{
    #[inline(always)]
    fn deserialize(deserializer: Deserializer<'de>) -> Result<Self, DeserializeError>
    where
        Self: Sized,
    {
        Ok(Box::new(<T as Deserialize<F>>::deserialize(deserializer)?))
    }

    #[inline(always)]
    fn deserialize_in_place(
        &mut self,
        deserializer: Deserializer<'de>,
    ) -> Result<(), DeserializeError> {
        <T as Deserialize<F>>::deserialize_in_place(self, deserializer)
    }
}

impl<T, F> Serialize<F> for Rc<T>
where
    F: BareFormula,
    T: SerializeRef<F>,
{
    #[inline(always)]
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        <T as SerializeRef<F>>::serialize(self.as_ref(), sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        <T as SerializeRef<F>>::size_hint(self.as_ref())
    }
}

impl<T, F> SerializeRef<F> for Rc<T>
where
    F: BareFormula,
    T: SerializeRef<F>,
{
    #[inline(always)]
    fn serialize<B>(&self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        <T as SerializeRef<F>>::serialize(self.as_ref(), sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        <T as SerializeRef<F>>::size_hint(self.as_ref())
    }
}

impl<'de, T, F> Deserialize<'de, F> for Rc<T>
where
    F: BareFormula,
    T: Deserialize<'de, F>,
{
    #[inline(always)]
    fn deserialize(deserializer: Deserializer<'de>) -> Result<Self, DeserializeError>
    where
        Self: Sized,
    {
        Ok(Rc::new(<T as Deserialize<F>>::deserialize(deserializer)?))
    }

    #[inline(always)]
    fn deserialize_in_place(
        &mut self,
        deserializer: Deserializer<'de>,
    ) -> Result<(), DeserializeError> {
        *self = Rc::new(<T as Deserialize<F>>::deserialize(deserializer)?);

        Ok(())
    }
}

impl<T, F> Serialize<F> for Arc<T>
where
    F: BareFormula,
    T: SerializeRef<F>,
{
    #[inline(always)]
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        <T as SerializeRef<F>>::serialize(self.as_ref(), sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        <T as SerializeRef<F>>::size_hint(self.as_ref())
    }
}

impl<T, F> SerializeRef<F> for Arc<T>
where
    F: BareFormula,
    T: SerializeRef<F>,
{
    #[inline(always)]
    fn serialize<B>(&self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        <T as SerializeRef<F>>::serialize(self.as_ref(), sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        <T as SerializeRef<F>>::size_hint(self.as_ref())
    }
}

impl<'de, T, F> Deserialize<'de, F> for Arc<T>
where
    F: BareFormula,
    T: Deserialize<'de, F>,
{
    #[inline(always)]
    fn deserialize(deserializer: Deserializer<'de>) -> Result<Self, DeserializeError>
    where
        Self: Sized,
    {
        Ok(Arc::new(<T as Deserialize<F>>::deserialize(deserializer)?))
    }

    #[inline(always)]
    fn deserialize_in_place(
        &mut self,
        deserializer: Deserializer<'de>,
    ) -> Result<(), DeserializeError> {
        *self = Arc::new(<T as Deserialize<F>>::deserialize(deserializer)?);

        Ok(())
    }
}
