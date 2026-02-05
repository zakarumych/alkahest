use crate::{
    Deserialize, DeserializeError, Deserializer,
    element::{Element, Indirect},
    list::List,
    serialize::{Serialize, Serializer, Sizes},
};

use alloc::vec::Vec;

impl<E, T> Serialize<List<E>> for Vec<T>
where
    E: Element,
    T: Serialize<E::Formula>,
{
    #[inline(always)]
    fn serialize<S>(&self, serializer: S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        Serialize::<List<E>>::serialize(&self[..], serializer)
    }

    #[inline(always)]
    fn size_hint<const SIZE_BYTES: usize>(&self) -> Option<Sizes> {
        Serialize::<List<E>>::size_hint::<SIZE_BYTES>(&self[..])
    }
}

impl<'de, E, T, const MIN: usize, const MAX: usize> Deserialize<'de, List<E, MIN, MAX>> for Vec<T>
where
    E: Element,
    T: Deserialize<'de, E::Formula>,
{
    #[inline(always)]
    fn deserialize<D>(deserializer: D) -> Result<Self, DeserializeError>
    where
        D: Deserializer<'de>,
    {
        let mut vec = Vec::new();
        Deserialize::<List<E, MIN, MAX>>::deserialize_in_place(&mut vec, deserializer)?;
        Ok(vec)
    }

    #[inline]
    fn deserialize_in_place<D>(&mut self, mut deserializer: D) -> Result<(), DeserializeError>
    where
        D: Deserializer<'de>,
    {
        let len = if MIN == MAX {
            debug_assert!(E::INHABITED || MIN == 0);
            MIN
        } else {
            let len = deserializer.read_usize()?;
            debug_assert!(E::INHABITED || len == 0);

            if len < MIN || len > MAX {
                return Err(DeserializeError::WrongLength);
            }

            len
        };

        if self.capacity() < len {
            self.reserve_exact(len - self.len());
        };

        let in_place = self.len().min(len);
        let extend = len - in_place;

        for i in 0..in_place {
            E::deserialize_in_place(&mut self[i], &mut deserializer)?;
        }
        for _ in 0..extend {
            self.push(E::deserialize(&mut deserializer)?);
        }

        Ok(())
    }
}

formula_alias!(for[E: Element] Vec<E> as Indirect<List<E>>);
