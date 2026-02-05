use crate::{
    deserialize::{Deserialize, DeserializeError, Deserializer, deserialize, deserialize_in_place},
    element::{Element, Indirect, stack_size},
    formula::SizeBound,
};

pub struct Lazy<'de, E> {
    input: &'de [u8],
    size_bytes: u8,
    element: core::marker::PhantomData<E>,
}

impl<'de, E> Lazy<'de, E> {
    #[inline]
    pub fn read<T>(&self) -> Result<T, DeserializeError>
    where
        E: Element,
        T: Deserialize<'de, E::Formula>,
    {
        match self.size_bytes {
            1 => deserialize::<E, T, 1>(self.input),
            2 => deserialize::<E, T, 2>(self.input),
            4 => deserialize::<E, T, 4>(self.input),
            8 => deserialize::<E, T, 8>(self.input),
            16 => deserialize::<E, T, 16>(self.input),
            _ => Err(DeserializeError::Incompatible),
        }
    }

    #[inline]
    pub fn read_in_place<T>(&self, place: &mut T) -> Result<(), DeserializeError>
    where
        E: Element,
        T: Deserialize<'de, E::Formula> + ?Sized,
    {
        match self.size_bytes {
            1 => deserialize_in_place::<E, T, 1>(place, self.input),
            2 => deserialize_in_place::<E, T, 2>(place, self.input),
            4 => deserialize_in_place::<E, T, 4>(place, self.input),
            8 => deserialize_in_place::<E, T, 8>(place, self.input),
            16 => deserialize_in_place::<E, T, 16>(place, self.input),
            _ => Err(DeserializeError::Incompatible),
        }
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
        let stack_size = match deserializer.size_bytes() {
            1 => stack_size::<E, 1>(),
            2 => stack_size::<E, 2>(),
            4 => stack_size::<E, 4>(),
            8 => stack_size::<E, 8>(),
            16 => stack_size::<E, 16>(),
            _ => return Err(DeserializeError::Incompatible),
        };

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
