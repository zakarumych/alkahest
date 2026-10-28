use core::marker::PhantomData;

use crate::{
    cold_err,
    deserialize::{
        ComplexDeserializer, Deserialize, DeserializeError, Deserializer, TrivialDeserializer,
        read_usize,
    },
    element::{Element, stack_size_versioned},
    formula::{Formula, SizeBound},
    list::List,
    serialize::trivial_padded_size,
};

/// A deferred read retaining both formula `F` and its selected version.
pub struct Lazy<'de, F: ?Sized, const VERSION: u32 = 0> {
    input: &'de [u8],
    size_bytes: usize,
    is_trivial: bool,
    marker: core::marker::PhantomData<F>,
}

impl<F: ?Sized, const VERSION: u32> Clone for Lazy<'_, F, VERSION> {
    #[inline]
    fn clone(&self) -> Self {
        Lazy {
            input: self.input,
            size_bytes: self.size_bytes,
            is_trivial: self.is_trivial,
            marker: PhantomData,
        }
    }
}

impl<'de, F, const VERSION: u32> Lazy<'de, F, VERSION>
where
    F: ?Sized,
{
    #[inline]
    pub fn read<T>(&self) -> Result<T, DeserializeError>
    where
        F: Formula<VERSION>,
        T: Deserialize<'de, F, VERSION>,
    {
        with_size_bytes!(SIZE_BYTES = self.size_bytes => {
            if self.is_trivial {
                T::deserialize(TrivialDeserializer::<SIZE_BYTES>::new(self.input))
            } else {
                T::deserialize(ComplexDeserializer::<SIZE_BYTES>::new(self.input))
            }
        } else {
            cold_err(DeserializeError::Incompatible)
        })
    }

    #[inline]
    pub fn read_in_place<T>(&self, place: &mut T) -> Result<(), DeserializeError>
    where
        F: Formula<VERSION>,
        T: Deserialize<'de, F, VERSION> + ?Sized,
    {
        with_size_bytes!(SIZE_BYTES = self.size_bytes => {
            if self.is_trivial {
                T::deserialize_in_place(place, TrivialDeserializer::<SIZE_BYTES>::new(self.input))
            } else {
                T::deserialize_in_place(place, ComplexDeserializer::<SIZE_BYTES>::new(self.input))
            }
        } else {
            cold_err(DeserializeError::Incompatible)
        })
    }

    fn read_usize(&mut self) -> Result<usize, DeserializeError> {
        with_size_bytes!(SIZE_BYTES = self.size_bytes => {{
            if self.input.len() < SIZE_BYTES {
                cold_err(DeserializeError::OutOfBounds(SIZE_BYTES))
            } else {
                let start = self.input.len() - SIZE_BYTES;
                let input = &self.input[start..];
                let input = input.as_array().unwrap();
                self.input = &self.input[..start];
                read_usize::<SIZE_BYTES>(input)
            }
        }} else {
            cold_err(DeserializeError::Incompatible)
        })
    }
}

impl<'de, E, const VERSION: u32> Lazy<'de, List<E>, VERSION>
where
    E: ?Sized,
{
    #[inline]
    pub fn iter<T>(
        &self,
    ) -> Result<impl Iterator<Item = Result<T, DeserializeError>>, DeserializeError>
    where
        E: Element<VERSION>,
        T: Deserialize<'de, E::Formula, VERSION>,
    {
        let mut me = self.clone();
        let len = simple_try!(me.read_usize());

        let input = with_size_bytes!(SIZE_BYTES = self.size_bytes => {{
            match const { trivial_padded_size::<E, SIZE_BYTES, VERSION>() } {
                Some(trivial_size) => {
                    let at = me.input.len() - trivial_size * len;
                    &me.input[at..]
                }
                None => me.input,
            }
        }} else {
            return cold_err(DeserializeError::Incompatible);
        });

        Ok(LazyDeIter::<'_, E, T, VERSION> {
            input,
            len,
            size_bytes: me.size_bytes,
            marker: PhantomData,
        })
    }
}

impl<'de, F, const VERSION: u32> Deserialize<'de, F, VERSION> for Lazy<'de, F, VERSION>
where
    F: Formula<VERSION> + ?Sized,
{
    #[inline]
    fn deserialize<D>(mut deserializer: D) -> Result<Self, DeserializeError>
    where
        D: Deserializer<'de>,
    {
        let input = deserializer.input();
        let size_bytes = deserializer.size_bytes();

        let stack = with_size_bytes!(SIZE_BYTES = size_bytes => {
            match const { stack_size_versioned::<F, SIZE_BYTES, VERSION>() } {
                SizeBound::Exact(size) => size,
                SizeBound::Bounded(size) => size.min(input.len()),
                SizeBound::Unbounded => input.len(),
            }
        } else {
            return cold_err(DeserializeError::Incompatible);
        });

        simple_try!(deserializer.read_bytes(stack));

        Ok(Lazy {
            input,
            size_bytes,
            is_trivial: D::IS_TRIVIAL,
            marker: core::marker::PhantomData,
        })
    }

    #[inline]
    fn deserialize_in_place<D>(&mut self, deserializer: D) -> Result<(), DeserializeError>
    where
        D: Deserializer<'de>,
    {
        *self = simple_try!(<Self as Deserialize<'de, F, VERSION>>::deserialize(
            deserializer
        ));
        Ok(())
    }
}

struct LazyDeIter<'de, E: ?Sized, T, const VERSION: u32 = 0> {
    input: &'de [u8],
    len: usize,
    size_bytes: usize,
    marker: PhantomData<fn(E) -> T>,
}

impl<'de, E, T, const VERSION: u32> Iterator for LazyDeIter<'de, E, T, VERSION>
where
    E: Element<VERSION> + ?Sized,
    T: Deserialize<'de, E::Formula, VERSION>,
{
    type Item = Result<T, DeserializeError>;

    fn next(&mut self) -> Option<Self::Item> {
        with_size_bytes!(SIZE_BYTES = self.size_bytes => {{
            if self.len == 0 {
                return None;
            }

            let (result, input) = match const { trivial_padded_size::<E, SIZE_BYTES, VERSION>() } {
                Some(trivial_size) => {
                    let at = self.input.len() - trivial_size;
                    let mut de = TrivialDeserializer::<SIZE_BYTES>::new(&self.input[at..]);
                    (E::deserialize::<T, _>(&mut de), &self.input[..at])
                }
                None => {
                    let mut de = ComplexDeserializer::<SIZE_BYTES>::new(self.input);
                    (E::deserialize::<T, _>(&mut de), de.input())
                }
            };

            match result {
                Ok(item) => {
                    self.len -= 1;
                    self.input = input;
                    Some(Ok(item))
                }
                Err(err) => {
                    self.len = 0;
                    self.input = &[];
                    Some(cold_err(err))
                }
            }
        }} else {
            Some(cold_err(DeserializeError::Incompatible))
        })
    }
}
