use crate::{
    deserialize::{Deserialize, DeserializeError, Deserializer, deserialize, deserialize_in_place},
    element::{Element, stack_size},
    formula::SizeBound,
};

pub struct Lazy<'de, E> {
    input: &'de [u8],
    size_bytes: usize,
    element: core::marker::PhantomData<E>,
}

impl<'de, E> Lazy<'de, E> {
    #[inline]
    pub fn read<T>(&self) -> Result<T, DeserializeError>
    where
        E: Element,
        T: Deserialize<'de, E::Formula>,
    {
        with_size_bytes!(SIZE_BYTES = self.size_bytes => {
            deserialize::<E, T, SIZE_BYTES>(self.input)
        } else {
            Err(DeserializeError::Incompatible)
        })
    }

    #[inline]
    pub fn read_in_place<T>(&self, place: &mut T) -> Result<(), DeserializeError>
    where
        E: Element,
        T: Deserialize<'de, E::Formula> + ?Sized,
    {
        with_size_bytes!(SIZE_BYTES = self.size_bytes => {
            deserialize_in_place::<E, T, SIZE_BYTES>(place, self.input)
        } else {
            Err(DeserializeError::Incompatible)
        })
    }
}

impl<'de, E> Deserialize<'de, E> for Lazy<'de, E>
where
    E: Element,
{
    #[inline]
    fn deserialize<D>(mut deserializer: D) -> Result<Self, DeserializeError>
    where
        D: Deserializer<'de>,
    {
        let stack_size = with_size_bytes!(SIZE_BYTES = deserializer.size_bytes() => {
            stack_size::<E, SIZE_BYTES>()
        } else {
            return Err(DeserializeError::Incompatible);
        });

        match stack_size {
            SizeBound::Exact(size) => {
                let input = deserializer.input();
                deserializer.read_bytes(size)?;
                Ok(Lazy {
                    input,
                    size_bytes: deserializer.size_bytes(),
                    element: core::marker::PhantomData,
                })
            }
            SizeBound::Bounded(0) => {
                let input = deserializer.input();
                Ok(Lazy {
                    input,
                    size_bytes: deserializer.size_bytes(),
                    element: core::marker::PhantomData,
                })
            }
            _ => Err(DeserializeError::Incompatible),
        }
    }

    #[inline(always)]
    fn deserialize_in_place<D>(&mut self, deserializer: D) -> Result<(), DeserializeError>
    where
        D: Deserializer<'de>,
    {
        *self = <Self as Deserialize<'de, E>>::deserialize(deserializer)?;
        Ok(())
    }
}
