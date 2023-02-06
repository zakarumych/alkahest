use alloc::vec::Vec;

use crate::{
    bytes::Bytes,
    // bytes::Bytes,
    deserialize::{Deserializer, Error, NonRefDeserialize},
    formula::Formula,
    reference::Ref,
    serialize::{NonRefSerializeOwned, Serializer},
};

impl<F> Formula for Vec<F>
where
    F: Formula,
{
    const MAX_SIZE: Option<usize> = <Ref<[F]> as Formula>::MAX_SIZE;

    type NonRef = [F];

    #[inline(always)]
    fn serialize<T, S>(value: T, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        T: NonRefSerializeOwned<[F]>,
        S: Serializer,
    {
        <Ref<[F]>>::serialize(value, ser)
    }

    #[inline(always)]
    fn deserialize<'de, T>(de: Deserializer<'de>) -> Result<T, Error>
    where
        T: NonRefDeserialize<'de, [F]>,
    {
        <Ref<[F]>>::deserialize(de)
    }

    #[inline(always)]
    fn deserialize_in_place<'de, T>(place: &mut T, de: Deserializer<'de>) -> Result<(), Error>
    where
        T: NonRefDeserialize<'de, [F]> + ?Sized,
    {
        <Ref<[F]>>::deserialize_in_place(place, de)
    }
}

impl<'de, F, T, const N: usize> NonRefDeserialize<'de, [F; N]> for Vec<T>
where
    F: Formula,
    T: NonRefDeserialize<'de, F::NonRef>,
{
    #[inline(always)]
    fn deserialize(de: Deserializer<'de>) -> Result<Self, Error> {
        de.into_iter::<F, T>()?.collect()
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

impl<'de, F, T> NonRefDeserialize<'de, [F]> for Vec<T>
where
    F: Formula,
    T: NonRefDeserialize<'de, F::NonRef>,
{
    #[inline(always)]
    fn deserialize(de: Deserializer<'de>) -> Result<Self, Error> {
        de.into_iter::<F, T>()?.collect()
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

impl<'de> NonRefDeserialize<'de, Bytes> for Vec<u8> {
    #[inline(always)]
    fn deserialize(de: Deserializer) -> Result<Self, Error> {
        Ok(de.read_all_bytes().to_vec())
    }

    #[inline(always)]
    fn deserialize_in_place(&mut self, de: Deserializer) -> Result<(), Error> {
        self.extend_from_slice(de.read_all_bytes());
        Ok(())
    }
}
