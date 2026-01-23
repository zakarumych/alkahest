use alloc::{boxed::Box, rc::Rc, sync::Arc};

use crate::{
    buffer::Buffer,
    deserialize::{Deserialize, DeserializeError, Deserializer},
    formula::BareFormulaType,
    serialize::{Serialize, Sizes},
    SerializeRef,
};

#[cfg(feature = "evolution")]
use crate::evolution::Descriptor;

impl<T, F> Serialize<F> for Box<T>
where
    F: BareFormulaType,
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
    F: BareFormulaType,
    T: ?Sized,
    for<'a> &'a T: Serialize<F>,
{
    #[inline(always)]
    fn serialize<B>(&self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        <&T as Serialize<F>>::serialize(self.as_ref(), sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        <&T as Serialize<F>>::size_hint(&self.as_ref())
    }
}

impl<'de, T, F> Deserialize<'de, F> for Box<T>
where
    F: BareFormulaType,
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
        Ok(Box::new(
            <T as Deserialize<F>>::deserialize_with_descriptor(descriptor, formula, deserializer)?,
        ))
    }

    #[cfg(feature = "evolution")]
    #[inline(always)]
    fn deserialize_in_place_with_descriptor(
        &mut self,
        descriptor: &Descriptor,
        formula: u32,
        deserializer: Deserializer<'de>,
    ) -> Result<(), DeserializeError> {
        <T as Deserialize<F>>::deserialize_in_place_with_descriptor(
            self,
            descriptor,
            formula,
            deserializer,
        )
    }
}

impl<T, F> Serialize<F> for Rc<T>
where
    F: BareFormulaType,
    T: ?Sized,
    for<'a> &'a T: Serialize<F>,
{
    #[inline(always)]
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        <&T as Serialize<F>>::serialize(self.as_ref(), sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        <&T as Serialize<F>>::size_hint(&self.as_ref())
    }
}

impl<T, F> SerializeRef<F> for Rc<T>
where
    F: BareFormulaType,
    T: ?Sized,
    for<'a> &'a T: Serialize<F>,
{
    #[inline(always)]
    fn serialize<B>(&self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        <&T as Serialize<F>>::serialize(self.as_ref(), sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        <&T as Serialize<F>>::size_hint(&self.as_ref())
    }
}

impl<'de, T, F> Deserialize<'de, F> for Rc<T>
where
    F: BareFormulaType,
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
        match Rc::get_mut(self) {
            None => *self = Rc::new(<T as Deserialize<F>>::deserialize(deserializer)?),
            Some(me) => <T as Deserialize<F>>::deserialize_in_place(me, deserializer)?,
        }

        Ok(())
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
        Ok(Rc::new(<T as Deserialize<F>>::deserialize_with_descriptor(
            descriptor,
            formula,
            deserializer,
        )?))
    }

    #[cfg(feature = "evolution")]
    #[inline(always)]
    fn deserialize_in_place_with_descriptor(
        &mut self,
        descriptor: &Descriptor,
        formula: u32,
        deserializer: Deserializer<'de>,
    ) -> Result<(), DeserializeError> {
        match Rc::get_mut(self) {
            None => {
                *self = Rc::new(<T as Deserialize<F>>::deserialize_with_descriptor(
                    descriptor,
                    formula,
                    deserializer,
                )?)
            }
            Some(me) => <T as Deserialize<F>>::deserialize_in_place_with_descriptor(
                me,
                descriptor,
                formula,
                deserializer,
            )?,
        }

        Ok(())
    }
}

impl<T, F> Serialize<F> for Arc<T>
where
    F: BareFormulaType,
    T: ?Sized,
    for<'a> &'a T: Serialize<F>,
{
    #[inline(always)]
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        <&T as Serialize<F>>::serialize(self.as_ref(), sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        <&T as Serialize<F>>::size_hint(&self.as_ref())
    }
}

impl<T, F> SerializeRef<F> for Arc<T>
where
    F: BareFormulaType,
    T: ?Sized,
    for<'a> &'a T: Serialize<F>,
{
    #[inline(always)]
    fn serialize<B>(&self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        <&T as Serialize<F>>::serialize(self.as_ref(), sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        <&T as Serialize<F>>::size_hint(&self.as_ref())
    }
}

impl<'de, T, F> Deserialize<'de, F> for Arc<T>
where
    F: BareFormulaType,
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
        Ok(Arc::new(
            <T as Deserialize<F>>::deserialize_with_descriptor(descriptor, formula, deserializer)?,
        ))
    }

    #[cfg(feature = "evolution")]
    #[inline(always)]
    fn deserialize_in_place_with_descriptor(
        &mut self,
        descriptor: &Descriptor,
        formula: u32,
        deserializer: Deserializer<'de>,
    ) -> Result<(), DeserializeError> {
        match Arc::get_mut(self) {
            None => {
                *self = Arc::new(<T as Deserialize<F>>::deserialize_with_descriptor(
                    descriptor,
                    formula,
                    deserializer,
                )?)
            }
            Some(me) => <T as Deserialize<F>>::deserialize_in_place_with_descriptor(
                me,
                descriptor,
                formula,
                deserializer,
            )?,
        }

        Ok(())
    }
}

#[cfg(feature = "derive")]
#[test]
pub fn test_box() {
    #[derive(alkahest_proc::Formula)]
    struct Foo {
        a: u32,
    }

    #[alkahest_proc::alkahest(SerializeRef<Foo>, Deserialize<'_, Foo>)]
    #[derive(Debug, PartialEq, Eq)]
    struct FooWithBox {
        a: Box<u32>,
    }

    let foo = FooWithBox { a: Box::new(42) };

    let mut buffer = [0u8; 4];
    crate::serialize::<Foo, _>(&foo, &mut buffer).unwrap();

    let foo2 = crate::deserialize::<Foo, FooWithBox>(&buffer).unwrap();

    assert_eq!(foo, foo2);
}
