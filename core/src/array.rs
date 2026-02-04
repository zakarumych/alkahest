use crate::{
    deserialize::{Deserialize, DeserializeError, Deserializer},
    element::{Element, heap_size, stack_size},
    formula::SizeBound,
    list::List,
    serialize::{Serialize, Serializer, Sizes},
};

impl<E, T, const N: usize, const MIN: usize, const MAX: usize> Serialize<List<E, MIN, MAX>>
    for [T; N]
where
    E: Element,
    T: Serialize<E::Formula>,
{
    #[inline]
    fn serialize<S>(&self, mut serializer: S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        const {
            assert!(N >= MIN && N <= MAX); // This should be trait bound, but it is not yet supported in Rust
            assert!(N == 0 || E::INHABITED);
        }

        for item in self {
            E::serialize(item, &mut serializer)?;
        }

        Ok(())
    }

    #[inline]
    fn size_hint<const SIZE_BYTES: u8>(&self) -> Option<Sizes> {
        const {
            assert!(N >= MIN && N <= MAX);
            assert!(N == 0 || E::INHABITED);
        }

        if N == 0 {
            return Some(Sizes::ZERO);
        }

        let mut sizes = Sizes::ZERO;

        match (stack_size::<E, SIZE_BYTES>(), heap_size::<E, SIZE_BYTES>()) {
            (SizeBound::Bounded(max_stack), SizeBound::Exact(heap_size)) => {
                // For heapless types, we can optimize size hint calculation
                // by using max stack size for all but the last element.
                // and adding size hint for the last element.

                sizes.add_stack((N - 1) * max_stack);
                sizes += self.last().unwrap().size_hint::<SIZE_BYTES>()?;
                sizes.add_heap(N * heap_size);
                Some(sizes)
            }
            (SizeBound::Exact(max_stack), SizeBound::Exact(heap_size)) => {
                // For heapless types, we can optimize size hint calculation
                // by using max stack size for all but the last element.
                // and adding size hint for the last element.

                sizes.add_stack(N * max_stack);
                sizes.add_heap(N * heap_size);
                Some(sizes)
            }
            _ => match N {
                // For short slices, just sum up size hints.
                0..4 => {
                    for item in self {
                        sizes += E::size_hint::<T, SIZE_BYTES>(item)?;
                    }
                    Some(sizes)
                }
                _ => None,
            },
        }
    }
}

impl<'de, E, T, const N: usize> Deserialize<'de, List<E, N, N>> for [T; N]
where
    E: Element,
    T: Deserialize<'de, E::Formula>,
{
    #[inline]
    fn deserialize<D>(mut deserializer: D) -> Result<Self, DeserializeError>
    where
        D: Deserializer<'de>,
    {
        const {
            assert!(N == 0 || E::INHABITED);
        }

        let mut options = [const { None::<T> }; N];

        for i in 0..N {
            match E::deserialize::<T, D>(&mut deserializer) {
                Ok(value) => {
                    options[i] = Some(value);
                }
                Err(err) => {
                    return Err(err);
                }
            }
        }

        // Unwrap is safe here because if loop above completed, all elements are `Some`.
        let array = options.map(|opt| opt.unwrap());
        Ok(array)
    }

    #[inline]
    fn deserialize_in_place<D>(&mut self, mut deserializer: D) -> Result<(), DeserializeError>
    where
        D: Deserializer<'de>,
    {
        const {
            assert!(N == 0 || E::INHABITED);
        }

        if N == 0 {
            return Ok(());
        }

        for i in 0..N {
            E::deserialize_in_place::<T, D>(&mut self[i], &mut deserializer)?;
        }

        Ok(())
    }
}
