use core::mem::size_of;

use alloc::collections::VecDeque;

use crate::{
    bytes::Bytes,
    deserialize::{Deserialize, DeserializeError, Deserializer},
    formula::Formula,
    iter::deserialize_extend_iter,
    reference::Ref,
    serialize::{Serialize, Serializer},
    size::FixedUsize,
    slice::{default_iter_fast_sizes_by_ref, default_iter_fast_sizes_owned},
};

impl<F> Formula for VecDeque<F>
where
    F: Formula,
{
    const MAX_STACK_SIZE: Option<usize> = <Ref<[F]> as Formula>::MAX_STACK_SIZE;
    const EXACT_SIZE: bool = <Ref<[F]> as Formula>::EXACT_SIZE;
    const HEAPLESS: bool = <Ref<[F]> as Formula>::HEAPLESS;
}

impl<F, T> Serialize<VecDeque<F>> for T
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
    fn size_hint(&self) -> Option<usize> {
        let size = self.size_hint()?;
        Some(size + size_of::<[FixedUsize; 2]>())
    }
}

impl<'de, F, T> Deserialize<'de, VecDeque<F>> for T
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

impl<F, T> Serialize<[F]> for VecDeque<T>
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
    fn size_hint(&self) -> Option<usize> {
        default_iter_fast_sizes_by_ref::<F, T, _>(self.iter())
    }
}

impl<'ser, F, T> Serialize<[F]> for &'ser VecDeque<T>
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
    fn size_hint(&self) -> Option<usize> {
        default_iter_fast_sizes_owned::<F, &'ser T, _>(self.iter())
    }
}

impl<'de, F, T> Deserialize<'de, [F]> for VecDeque<T>
where
    F: Formula,
    T: Deserialize<'de, F>,
{
    #[inline(always)]
    fn deserialize(de: Deserializer<'de>) -> Result<Self, DeserializeError> {
        de.into_unsized_iter::<F, T>()?.collect()
    }

    #[inline(always)]
    fn deserialize_in_place(&mut self, de: Deserializer<'de>) -> Result<(), DeserializeError> {
        self.clear();
        deserialize_extend_iter::<F, T, Self>(self, de)
    }
}

impl<'de, F, T, const N: usize> Deserialize<'de, [F; N]> for VecDeque<T>
where
    F: Formula,
    T: Deserialize<'de, F>,
{
    #[inline(always)]
    fn deserialize(de: Deserializer<'de>) -> Result<Self, DeserializeError> {
        de.into_unsized_iter::<F, T>()?.collect()
    }

    #[inline(always)]
    fn deserialize_in_place(&mut self, de: Deserializer<'de>) -> Result<(), DeserializeError> {
        self.clear();
        deserialize_extend_iter::<F, T, Self>(self, de)
    }
}

impl Serialize<Bytes> for VecDeque<u8> {
    #[inline(always)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut ser = ser.into();
        let (head, tail) = self.as_slices();
        ser.write_bytes(tail)?;
        ser.write_bytes(head)?;
        ser.finish()
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<usize> {
        Some(self.len())
    }
}

impl Serialize<Bytes> for &VecDeque<u8> {
    #[inline(always)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut ser = ser.into();
        let (head, tail) = self.as_slices();
        ser.write_bytes(tail)?;
        ser.write_bytes(head)?;
        ser.finish()
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<usize> {
        Some(self.len())
    }
}

impl<'de> Deserialize<'de, Bytes> for VecDeque<u8> {
    #[inline(always)]
    fn deserialize(de: Deserializer) -> Result<Self, DeserializeError> {
        let bytes = de.read_all_bytes();
        let mut deque = VecDeque::with_capacity(bytes.len());
        deque.extend(bytes);
        Ok(deque)
    }

    #[inline(always)]
    fn deserialize_in_place(&mut self, de: Deserializer) -> Result<(), DeserializeError> {
        self.clear();
        self.extend(de.read_all_bytes());
        Ok(())
    }
}
