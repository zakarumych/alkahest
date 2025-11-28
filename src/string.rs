use alloc::{borrow::ToOwned, string::String};

use crate::{
    buffer::Buffer,
    deserialize::{Deserialize, DeserializeError, Deserializer},
    formula::{reference_size, Formula},
    reference::Ref,
    serialize::{write_bytes, write_ref, write_reference, Serialize, Sizes}, SerializeRef,
};

impl Formula for String {
    const MAX_STACK_SIZE: Option<usize> = <Ref<str> as Formula>::MAX_STACK_SIZE;
    const EXACT_SIZE: bool = <Ref<str> as Formula>::EXACT_SIZE;
    const HEAPLESS: bool = <Ref<str> as Formula>::HEAPLESS;
}

impl<T> Serialize<String> for T
where
    T: Serialize<str>,
{
    #[inline(always)]
    fn serialize<B>(self, sizes: &mut Sizes, mut buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        let size = write_ref::<str, T, _>(self, sizes, buffer.reborrow())?;
        write_reference::<str, B>(size, sizes.heap, sizes.heap, sizes.stack, buffer)?;
        sizes.stack += reference_size::<str>();
        Ok(())
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        let mut sizes = <Self as Serialize<str>>::size_hint(self)?;
        sizes.to_heap(0);
        sizes.add_stack(reference_size::<str>());
        Some(sizes)
    }
}

impl<T> SerializeRef<String> for T
where
    T: ?Sized,
    for<'a> &'a T: Serialize<str>,
{
    #[inline(always)]
    fn serialize<B>(&self, sizes: &mut Sizes, mut buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        let size = write_ref::<str, &T, _>(self, sizes, buffer.reborrow())?;
        write_reference::<str, B>(size, sizes.heap, sizes.heap, sizes.stack, buffer)?;
        sizes.stack += reference_size::<str>();
        Ok(())
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        let mut sizes = <&Self as Serialize<str>>::size_hint(&self)?;
        sizes.to_heap(0);
        sizes.add_stack(reference_size::<str>());
        Some(sizes)
    }
}

impl<'de, T> Deserialize<'de, String> for T
where
    T: Deserialize<'de, str>,
{
    #[inline(always)]
    fn deserialize(de: Deserializer<'de>) -> Result<T, DeserializeError> {
        let de = de.deref::<str>()?;
        <T as Deserialize<str>>::deserialize(de)
    }

    #[inline(always)]
    fn deserialize_in_place(&mut self, de: Deserializer<'de>) -> Result<(), DeserializeError> {
        let de = de.deref::<str>()?;
        <T as Deserialize<str>>::deserialize_in_place(self, de)
    }
}

impl Serialize<str> for String {
    #[inline(always)]
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        write_bytes(self.as_bytes(), sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        Some(Sizes::with_stack(self.len()))
    }
}

impl Serialize<str> for &String {
    #[inline(always)]
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        write_bytes(self.as_bytes(), sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        Some(Sizes::with_stack(self.len()))
    }
}

impl<'de> Deserialize<'de, str> for String {
    #[inline(always)]
    fn deserialize(deserializer: Deserializer<'de>) -> Result<Self, DeserializeError> {
        let string = <&str as Deserialize<'de, str>>::deserialize(deserializer)?;
        Ok(string.to_owned())
    }

    #[inline(always)]
    fn deserialize_in_place(
        &mut self,
        deserializer: Deserializer<'de>,
    ) -> Result<(), DeserializeError> {
        self.clear();
        let string = <&str as Deserialize<'de, str>>::deserialize(deserializer)?;
        self.push_str(string);
        Ok(())
    }
}
