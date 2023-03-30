use alloc::vec::Vec;

use crate::{
    buffer::Buffer,
    bytes::Bytes,
    deserialize::{Deserialize, DeserializeError, Deserializer},
    formula::{reference_size, Formula},
    iter::{deserialize_extend_iter, owned_iter_fast_sizes, ref_iter_fast_sizes},
    reference::Ref,
    serialize::{write_bytes, write_ref, write_slice, Serialize, Sizes},
};

impl<F> Formula for Vec<F>
where
    F: Formula,
{
    const MAX_STACK_SIZE: Option<usize> = <Ref<[F]> as Formula>::MAX_STACK_SIZE;
    const EXACT_SIZE: bool = <Ref<[F]> as Formula>::EXACT_SIZE;
    const HEAPLESS: bool = <Ref<[F]> as Formula>::HEAPLESS;
}

impl<F, T> Serialize<Vec<F>> for T
where
    F: Formula,
    T: Serialize<[F]>,
{
    #[inline(always)]
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        write_ref::<[F], T, B>(self, sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        let mut sizes = <Self as Serialize<[F]>>::size_hint(self)?;
        sizes.to_heap(0);
        sizes.add_stack(reference_size::<[F]>());
        Some(sizes)
    }
}

impl<'de, F, T> Deserialize<'de, Vec<F>> for T
where
    F: Formula,
    T: Deserialize<'de, [F]>,
{
    #[inline(always)]
    fn deserialize(de: Deserializer<'de>) -> Result<T, DeserializeError> {
        let de = de.deref::<[F]>()?;
        <T as Deserialize<[F]>>::deserialize(de)
    }

    #[inline(always)]
    fn deserialize_in_place(&mut self, de: Deserializer<'de>) -> Result<(), DeserializeError> {
        let de = de.deref::<[F]>()?;
        <T as Deserialize<[F]>>::deserialize_in_place(self, de)
    }
}

impl<F, T> Serialize<[F]> for Vec<T>
where
    F: Formula,
    T: Serialize<F>,
{
    #[inline(always)]
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        write_slice(self.into_iter(), sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        owned_iter_fast_sizes::<F, _, _>(self.iter())
    }
}

impl<'ser, F, T> Serialize<[F]> for &'ser Vec<T>
where
    F: Formula,
    &'ser T: Serialize<F>,
{
    #[inline(always)]
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        write_slice(self.iter(), sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        ref_iter_fast_sizes::<F, _, _>(self.iter())
    }
}

impl<'de, F, T> Deserialize<'de, [F]> for Vec<T>
where
    F: Formula,
    T: Deserialize<'de, F>,
{
    #[inline(always)]
    fn deserialize(de: Deserializer<'de>) -> Result<Self, DeserializeError> {
        de.into_unsized_iter::<F, T>().collect()
    }

    #[inline(always)]
    fn deserialize_in_place(&mut self, de: Deserializer<'de>) -> Result<(), DeserializeError> {
        self.clear();
        deserialize_extend_iter::<F, T, Self>(self, de)
    }
}

impl<'de, F, T, const N: usize> Deserialize<'de, [F; N]> for Vec<T>
where
    F: Formula,
    T: Deserialize<'de, F>,
{
    #[inline(always)]
    fn deserialize(de: Deserializer<'de>) -> Result<Self, DeserializeError> {
        de.into_unsized_iter::<F, T>().collect()
    }

    #[inline(always)]
    fn deserialize_in_place(&mut self, de: Deserializer<'de>) -> Result<(), DeserializeError> {
        self.clear();
        deserialize_extend_iter::<F, T, Self>(self, de)
    }
}

impl Serialize<Bytes> for Vec<u8> {
    #[inline(always)]
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        write_bytes(self.as_slice(), sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        Some(Sizes::with_stack(self.len()))
    }
}

impl Serialize<Bytes> for &Vec<u8> {
    #[inline(always)]
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        write_bytes(self.as_slice(), sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        Some(Sizes::with_stack(self.len()))
    }
}

impl<'de> Deserialize<'de, Bytes> for Vec<u8> {
    #[inline(always)]
    fn deserialize(de: Deserializer) -> Result<Self, DeserializeError> {
        let mut vec = Vec::new();
        vec.extend_from_slice(de.read_all_bytes());
        Ok(vec)
    }

    #[inline(always)]
    fn deserialize_in_place(&mut self, de: Deserializer) -> Result<(), DeserializeError> {
        self.clear();
        self.extend_from_slice(de.read_all_bytes());
        Ok(())
    }
}
