use alloc::vec::Vec;

use crate::{
    bytes::Bytes,
    deserialize::{Deserialize, DeserializeError, Deserializer},
    formula::Formula,
    iter::deserialize_extend_iter,
    reference::Ref,
    serialize::{reference_size, Serialize, Serializer},
    slice::default_iter_fast_sizes,
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
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        T: Serialize<[F]>,
        S: Serializer,
    {
        ser.into().write_ref::<[F], T>(self)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<(usize, usize)> {
        let (heap, stack) = <Self as Serialize<[F]>>::size_hint(self)?;
        Some((heap + stack, reference_size::<[F]>()))
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
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut ser = ser.into();
        ser.write_slice(self.into_iter())?;
        ser.finish()
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<(usize, usize)> {
        Some((0, default_iter_fast_sizes::<F, _>(&self.iter())?))
    }
}

impl<'ser, F, T> Serialize<[F]> for &'ser Vec<T>
where
    F: Formula,
    &'ser T: Serialize<F>,
{
    #[inline(always)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut ser = ser.into();
        ser.write_slice(self.iter())?;
        ser.finish()
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<(usize, usize)> {
        Some((0, default_iter_fast_sizes::<F, _>(&self.iter())?))
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
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Serialize::<Bytes>::serialize(&*self, ser)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<(usize, usize)> {
        Some((0, self.len()))
    }
}

impl Serialize<Bytes> for &Vec<u8> {
    #[inline(always)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut ser = ser.into();
        ser.write_bytes(self)?;
        ser.finish()
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<(usize, usize)> {
        Some((0, self.len()))
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
