use crate::{
    Deserialize, DeserializeError, Deserializer, cold_err,
    element::{Element, Indirect, stack_size},
    formula::SizeBound,
    list::List,
    serialize::{Serialize, Serializer, Sizes},
};

use alloc::vec::Vec;

fn validate_length<'de, E: Element + ?Sized, D: Deserializer<'de>>(
    deserializer: &D,
    len: usize,
) -> Result<(), DeserializeError> {
    with_size_bytes!(SIZE_BYTES = deserializer.size_bytes() => {
        {
            let required = match stack_size::<E, SIZE_BYTES>() {
                SizeBound::Exact(size) => size.checked_mul(len),
                // Only the final element may omit trailing padding.
                SizeBound::Bounded(size) => size.checked_mul(len.saturating_sub(1)),
                SizeBound::Unbounded => Some(0),
            };
            match required {
                Some(required) if required <= deserializer.input().len() => Ok(()),
                _ => cold_err(DeserializeError::WrongLength),
            }
        }
    } else {
        cold_err(DeserializeError::Incompatible)
    })
}

impl<E, T> Serialize<List<E>> for Vec<T>
where
    E: Element + ?Sized,
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
    E: Element + ?Sized,
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
                return cold_err(DeserializeError::WrongLength);
            }

            len
        };

        simple_try!(validate_length::<E, _>(&deserializer, len));

        simple_try!(
            vec.try_reserve(len)
                .map_err(|_| DeserializeError::WrongLength)
        );

        for _ in 0..len {
            let value = simple_try!(E::deserialize(&mut deserializer));
            vec.push(value);
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
                return cold_err(DeserializeError::WrongLength);
            }

            len
        };

        simple_try!(validate_length::<E, _>(&deserializer, len));

        simple_try!(
            self.try_reserve(len.saturating_sub(self.len()))
                .map_err(|_| DeserializeError::WrongLength)
        );

        let in_place = self.len().min(len);
        let extend = len - in_place;

        for i in 0..in_place {
            simple_try!(E::deserialize_in_place(&mut self[i], &mut deserializer));
        }
        for _ in 0..extend {
            let value = simple_try!(E::deserialize(&mut deserializer));
            self.push(value);
        }

        self.truncate(len);

        Ok(())
    }
}

// Vec is commonly used in compound types,
// so the alias makes it indirect.
element_alias!(for[E: Element] Vec<E> as Indirect<List<E>>);
