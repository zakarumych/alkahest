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
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        Serialize::<List<E>>::serialize(&self[..], serializer)
    }

    #[inline]
    fn size_hint<const SIZE_BYTES: usize>(&self) -> Option<Sizes> {
        Serialize::<List<E>>::size_hint::<SIZE_BYTES>(&self[..])
    }
}

impl<'de, E, T, const MIN: usize, const MAX: usize> Deserialize<'de, List<E, MIN, MAX>> for Vec<T>
where
    E: Element,
    T: Deserialize<'de, E::Formula>,
{
    #[inline]
    fn deserialize<D>(mut deserializer: D) -> Result<Self, DeserializeError>
    where
        D: Deserializer<'de>,
    {
        let mut vec = Vec::new();
        let len = if MIN == MAX {
            debug_assert!(E::INHABITED || MIN == 0);
            MIN
        } else {
            let len = simple_try!(deserializer.read_usize());
            debug_assert!(E::INHABITED || len == 0);

            if len < MIN || len > MAX {
                return Err(DeserializeError::WrongLength);
            }

            len
        };

        vec.reserve_exact(len);

        for _ in 0..len {
            vec.push(simple_try!(E::deserialize(&mut deserializer)));
        }

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
            let len = simple_try!(deserializer.read_usize());
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
            simple_try!(E::deserialize_in_place(&mut self[i], &mut deserializer));
        }
        for _ in 0..extend {
            self.push(simple_try!(E::deserialize(&mut deserializer)));
        }

        Ok(())
    }
}

// Vec is commonly used in compound types,
// so the alias makes it indirect.
formula_alias!(for[E: Element] Vec<E> as Indirect<List<E>>);
