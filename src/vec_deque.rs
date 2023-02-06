use alloc::collections::VecDeque;

use crate::{
    bytes::Bytes,
    deserialize::{Deserialize, Deserializer, Error},
    formula::Formula,
    reference::Ref,
    serialize::{SerializeOwned, Serializer},
    Serialize,
};

impl<'de, F, T, const N: usize> Deserialize<'de, [F; N]> for VecDeque<T>
where
    F: Formula,
    T: Deserialize<'de, F::NonRef>,
{
    #[cfg_attr(feature = "inline-more", inline(always))]
    fn deserialize(de: Deserializer<'de>) -> Result<Self, Error> {
        de.into_iter::<F, T>()?.collect()
    }

    #[cfg_attr(feature = "inline-more", inline(always))]
    fn deserialize_in_place(&mut self, de: Deserializer<'de>) -> Result<(), Error> {
        let iter = de.into_iter::<F, T>()?;
        self.reserve(iter.len());
        for elem in iter {
            self.push_back(elem?);
        }
        Ok(())
    }
}

impl<F, T> SerializeOwned<[F]> for VecDeque<T>
where
    T: SerializeOwned<F::NonRef>,
    F: Formula,
{
    fn serialize_owned<S>(self, er: impl Into<S>) -> Result<S::Ok, S::Error>
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

impl<F, T> Serialize<[F]> for VecDeque<T>
where
    T: Serialize<F::NonRef>,
    F: Formula,
{
    fn serialize<S>(&self, er: impl Into<S>) -> Result<S::Ok, S::Error>
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
    T: Deserialize<'de, F::NonRef>,
{
    #[cfg_attr(feature = "inline-more", inline(always))]
    fn deserialize(de: Deserializer<'de>) -> Result<Self, Error> {
        de.into_iter::<F, T>()?.collect()
    }

    #[cfg_attr(feature = "inline-more", inline(always))]
    fn deserialize_in_place(&mut self, de: Deserializer<'de>) -> Result<(), Error> {
        let iter = de.into_iter::<F, T>()?;
        self.reserve(iter.len());
        for elem in iter {
            self.push_back(elem?);
        }
        Ok(())
    }
}

impl SerializeOwned<Bytes> for VecDeque<u8> {
    #[cfg_attr(feature = "inline-more", inline(always))]
    fn serialize_owned<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Serialize::<Bytes>::serialize(&self, ser)
    }
}

impl Serialize<Bytes> for VecDeque<u8> {
    #[cfg_attr(feature = "inline-more", inline(always))]
    fn serialize<S>(&self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
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
    #[cfg_attr(feature = "inline-more", inline(always))]
    fn deserialize(de: Deserializer) -> Result<Self, Error> {
        let mut deque = VecDeque::new();
        deque.extend(de.read_all_bytes());
        Ok(deque)
    }

    #[cfg_attr(feature = "inline-more", inline(always))]
    fn deserialize_in_place(&mut self, de: Deserializer) -> Result<(), Error> {
        self.extend(de.read_all_bytes());
        Ok(())
    }
}
