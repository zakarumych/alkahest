use core::mem::size_of;

use alloc::vec::Vec;

use crate::{
    bytes::Bytes,
    deserialize::{Deserialize, Deserializer, Error},
    formula::Formula,
    reference::Ref,
    serialize::{Serialize, Serializer},
    slice::{default_iter_fast_sizes_by_ref, default_iter_fast_sizes_owned},
    FixedUsize,
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
        let mut ser = ser.into();
        ser.write_ref::<[F], T>(self)?;
        ser.finish()
    }

    #[inline(always)]
    fn fast_sizes(&self) -> Option<usize> {
        let size = self.fast_sizes()?;
        Some(size + size_of::<[FixedUsize; 2]>())
    }
}

impl<'de, F, T> Deserialize<'de, Vec<F>> for T
where
    F: Formula,
    T: Deserialize<'de, [F]>,
{
    #[inline(always)]
    fn deserialize(de: Deserializer<'de>) -> Result<T, Error> {
        let de = de.deref()?;
        <T as Deserialize<[F]>>::deserialize(de)
    }

    #[inline(always)]
    fn deserialize_in_place(&mut self, de: Deserializer<'de>) -> Result<(), Error> {
        let de = de.deref()?;
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
        self.into_iter()
            .try_for_each(|elem| ser.write_value::<F, T>(elem))?;
        ser.finish()
    }

    #[inline(always)]
    fn fast_sizes(&self) -> Option<usize> {
        default_iter_fast_sizes_by_ref::<F, T, _>(self.iter())
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
        self.into_iter()
            .try_for_each(|elem| ser.write_value::<F, &'ser T>(elem))?;
        ser.finish()
    }

    #[inline(always)]
    fn fast_sizes(&self) -> Option<usize> {
        default_iter_fast_sizes_owned::<F, &'ser T, _>(self.iter())
    }
}

impl<'de, F, T> Deserialize<'de, [F]> for Vec<T>
where
    F: Formula,
    T: Deserialize<'de, F>,
{
    #[inline(always)]
    fn deserialize(de: Deserializer<'de>) -> Result<Self, Error> {
        let iter = de.into_iter::<F, T>()?;
        let mut vec = Vec::with_capacity(iter.len());
        for elem in iter {
            vec.push(elem?);
        }
        Ok(vec)
    }

    #[inline(always)]
    fn deserialize_in_place(&mut self, de: Deserializer<'de>) -> Result<(), Error> {
        let iter = de.into_iter::<F, T>()?;
        self.reserve(iter.len());
        for elem in iter {
            self.push(elem?);
        }
        Ok(())
    }
}

impl<'de, F, T, const N: usize> Deserialize<'de, [F; N]> for Vec<T>
where
    F: Formula,
    T: Deserialize<'de, F>,
{
    #[inline(always)]
    fn deserialize(de: Deserializer<'de>) -> Result<Self, Error> {
        let iter = de.into_iter_hint::<F, T>(N)?;
        let mut vec = Vec::with_capacity(iter.len());
        for elem in iter {
            vec.push(elem?);
        }
        Ok(vec)
    }

    #[inline(always)]
    fn deserialize_in_place(&mut self, de: Deserializer<'de>) -> Result<(), Error> {
        let iter = de.into_iter_hint::<F, T>(N)?;
        self.reserve(iter.len());
        for elem in iter {
            self.push(elem?);
        }
        Ok(())
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
    fn fast_sizes(&self) -> Option<usize> {
        Some(self.len())
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
    fn fast_sizes(&self) -> Option<usize> {
        Some(self.len())
    }
}

impl<'de> Deserialize<'de, Bytes> for Vec<u8> {
    #[inline(always)]
    fn deserialize(de: Deserializer) -> Result<Self, Error> {
        let mut vec = Vec::new();
        vec.extend(de.read_all_bytes());
        Ok(vec)
    }

    #[inline(always)]
    fn deserialize_in_place(&mut self, de: Deserializer) -> Result<(), Error> {
        self.extend(de.read_all_bytes());
        Ok(())
    }
}
