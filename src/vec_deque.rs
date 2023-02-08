use alloc::collections::VecDeque;

use crate::{
    bytes::Bytes,
    deserialize::{Deserialize, Deserializer, Error},
    formula::Formula,
    reference::Ref,
    serialize::{Serialize, Serializer},
};

impl<F> Formula for VecDeque<F>
where
    F: Formula,
{
    const MAX_STACK_SIZE: Option<usize> = <Ref<[F]> as Formula>::MAX_STACK_SIZE;
    const EXACT_SIZE: bool = <Ref<[F]> as Formula>::EXACT_SIZE;
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
        <T as Serialize<Ref<[F]>>>::serialize(self, ser)
    }
}

impl<'de, F, T> Deserialize<'de, VecDeque<F>> for T
where
    F: Formula,
    T: Deserialize<'de, [F]>,
{
    #[inline(always)]
    fn deserialize(de: Deserializer<'de>) -> Result<T, Error> {
        <T as Deserialize<'de, Ref<[F]>>>::deserialize(de)
    }

    #[inline(always)]
    fn deserialize_in_place(&mut self, de: Deserializer<'de>) -> Result<(), Error> {
        <T as Deserialize<'de, Ref<[F]>>>::deserialize_in_place(self, de)
    }
}

impl<'de, F, T, const N: usize> Deserialize<'de, [F; N]> for VecDeque<T>
where
    F: Formula,
    T: Deserialize<'de, F>,
{
    #[inline(always)]
    fn deserialize(de: Deserializer<'de>) -> Result<Self, Error> {
        let mut deque = VecDeque::new();
        Deserialize::<[F]>::deserialize_in_place(&mut deque, de)?;
        Ok(deque)
    }

    #[inline(always)]
    fn deserialize_in_place(&mut self, de: Deserializer<'de>) -> Result<(), Error> {
        let iter = de.into_iter::<F, T>()?;
        self.reserve(iter.len());
        for elem in iter {
            self.push_back(elem?);
        }
        Ok(())
    }
}

impl<'de, F, T> Deserialize<'de, [F]> for VecDeque<T>
where
    F: Formula,
    T: Deserialize<'de, F>,
{
    #[inline(always)]
    fn deserialize(de: Deserializer<'de>) -> Result<Self, Error> {
        let mut deque = VecDeque::new();
        Deserialize::<[F]>::deserialize_in_place(&mut deque, de)?;
        Ok(deque)
    }

    #[inline(always)]
    fn deserialize_in_place(&mut self, de: Deserializer<'de>) -> Result<(), Error> {
        let iter = de.into_iter::<F, T>()?;
        self.reserve(iter.len());
        for elem in iter {
            self.push_back(elem?);
        }
        Ok(())
    }
}

impl Serialize<Bytes> for VecDeque<u8> {
    #[inline(always)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Serialize::<Bytes>::serialize(&self, ser)
    }

    #[inline(always)]
    fn fast_sizes(&self) -> Option<(usize, usize)> {
        Some((0, self.len()))
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
        ser.write_bytes(head)?;
        ser.write_bytes(tail)?;
        ser.finish()
    }

    #[inline(always)]
    fn fast_sizes(&self) -> Option<(usize, usize)> {
        Some((0, self.len()))
    }
}

impl<'de> Deserialize<'de, Bytes> for VecDeque<u8> {
    #[inline(always)]
    fn deserialize(de: Deserializer) -> Result<Self, Error> {
        let mut deque = VecDeque::new();
        deque.extend(de.read_all_bytes());
        Ok(deque)
    }

    #[inline(always)]
    fn deserialize_in_place(&mut self, de: Deserializer) -> Result<(), Error> {
        self.extend(de.read_all_bytes());
        Ok(())
    }
}
