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
    const MAX_SIZE: Option<usize> = <Ref<[F]> as Formula>::MAX_SIZE;

    type NonRef = [F];
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
        de.into_iter::<F, T>()?.collect()
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

impl<F, T> Serialize<[F]> for VecDeque<T>
where
    T: Serialize<F>,
    F: Formula,
{
    fn serialize<S>(self, er: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut er = er.into();
        for elem in self {
            er.write_value::<F, _>(elem)?;
        }
        er.finish()
    }
}

impl<'ser, F, T> Serialize<[F]> for &'ser VecDeque<T>
where
    &'ser T: Serialize<F>,
    F: Formula,
{
    fn serialize<S>(self, er: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut er = er.into();
        for elem in self {
            er.write_value::<F, _>(elem)?;
        }
        er.finish()
    }
}

impl<'de, F, T> Deserialize<'de, [F]> for VecDeque<T>
where
    F: Formula,
    T: Deserialize<'de, F>,
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