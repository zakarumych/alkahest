use core::marker::PhantomData;

use crate::{
    cold_err,
    deserialize::{
        ComplexDeserializer, Deserialize, DeserializeError, Deserializer, TrivialDeserializer,
        read_usize,
    },
    element::{Element, stack_size},
    formula::{Formula, SizeBound},
    list::List,
};

pub struct Lazy<'de, F: ?Sized> {
    input: &'de [u8],
    size_bytes: usize,
    is_trivial: bool,
    marker: core::marker::PhantomData<F>,
}

impl<F: ?Sized> Clone for Lazy<'_, F> {
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

impl<'de, F> Lazy<'de, F>
where
    F: ?Sized,
{
    #[inline]
    pub fn read<T>(&self) -> Result<T, DeserializeError>
    where
        F: Formula,
        T: Deserialize<'de, F>,
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
        F: Formula,
        T: Deserialize<'de, F> + ?Sized,
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

impl<'de, E> Lazy<'de, List<E>>
where
    E: ?Sized,
{
    #[inline]
    pub fn iter<T>(
        &self,
    ) -> Result<impl Iterator<Item = Result<T, DeserializeError>>, DeserializeError>
    where
        E: Element,
        T: Deserialize<'de, E::Formula>,
    {
        let mut me = self.clone();
        let len = simple_try!(me.read_usize());

        Ok(LazyDeIter::<'_, E, T> {
            input: me.input,
            len,
            size_bytes: me.size_bytes,
            marker: PhantomData,
        })
    }
}

impl<'de, F> Deserialize<'de, F> for Lazy<'de, F>
where
    F: Formula + ?Sized,
{
    #[inline]
    fn deserialize<D>(mut deserializer: D) -> Result<Self, DeserializeError>
    where
        D: Deserializer<'de>,
    {
        let input = deserializer.input();
        let size_bytes = deserializer.size_bytes();
        let stack = with_size_bytes!(SIZE_BYTES = size_bytes => {
            match const { stack_size::<F, SIZE_BYTES>() } {
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
        *self = simple_try!(<Self as Deserialize<'de, F>>::deserialize(deserializer));
        Ok(())
    }
}

struct LazyDeIter<'de, E: ?Sized, T> {
    input: &'de [u8],
    len: usize,
    size_bytes: usize,
    marker: PhantomData<fn(E) -> T>,
}

impl<'de, E, T> Iterator for LazyDeIter<'de, E, T>
where
    E: Element + ?Sized,
    T: Deserialize<'de, E::Formula>,
{
    type Item = Result<T, DeserializeError>;

    fn next(&mut self) -> Option<Self::Item> {
        with_size_bytes!(SIZE_BYTES = self.size_bytes => {{
            if self.len == 0 {
                return None;
            }

            let mut de = ComplexDeserializer::<SIZE_BYTES>::new(self.input);
            match E::deserialize::<T, _>(&mut de) {
                Ok(item) => {
                    self.len -= 1;
                    self.input = de.input();
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
