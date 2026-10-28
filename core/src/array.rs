use crate::{
    advanced::{size_hint_padded_versioned, size_hint_versioned},
    deserialize::{Deserialize, DeserializeError, Deserializer},
    element::{Element, heap_size_versioned, stack_size_versioned},
    formula::SizeBound,
    list::List,
    serialize::{Serialize, Serializer, Sizes},
};

// Array of `N` elements of type `T` can be serialized as a list of `N` elements of type `E` if `T` can be serialized as `E`,
// and N is within the bounds of the list formula.
impl<E, T, const N: usize, const MIN: usize, const MAX: usize, const VERSION: u32>
    Serialize<List<E, MIN, MAX>, VERSION> for [T; N]
where
    E: Element<VERSION> + ?Sized,
    T: Serialize<E::Formula, VERSION>,
{
    #[inline]
    fn serialize<S>(&self, mut serializer: S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        const {
            assert!(N >= MIN && N <= MAX); // This should be trait bound, but it is not yet supported in Rust.
            assert!(N == 0 || E::INHABITED); // Either empty or inhabited element type.
        }

        if MIN != MAX && E::INHABITED {
            simple_try!(serializer.write_usize(N));
        }

        E::serialize_slice::<T, S>(self, &mut serializer)
    }

    #[inline]
    fn size_hint<const SIZE_BYTES: usize>(&self) -> Option<Sizes> {
        const {
            assert!(N >= MIN && N <= MAX);
            assert!(N == 0 || E::INHABITED);
        }

        let mut sizes = if MIN != MAX && E::INHABITED {
            Sizes::with_stack(SIZE_BYTES)
        } else {
            Sizes::ZERO
        };

        if N == 0 {
            return Some(sizes);
        }

        match const {
            (
                stack_size_versioned::<E, SIZE_BYTES, VERSION>(),
                heap_size_versioned::<E, SIZE_BYTES, VERSION>(),
            )
        } {
            (SizeBound::Bounded(max_stack), SizeBound::Exact(heap_size)) => {
                sizes.add_stack((N - 1) * max_stack);
                sizes.add_heap(N * heap_size);
                sizes.stack += simple_some!(size_hint_versioned::<E, _, SIZE_BYTES, VERSION>(
                    self.last().unwrap()
                ))
                .stack;
                Some(sizes)
            }
            (SizeBound::Exact(max_stack), SizeBound::Exact(heap_size)) => {
                sizes.add_stack(N * max_stack);
                sizes.add_heap(N * heap_size);
                Some(sizes)
            }
            _ => match N {
                // For short slices, just sum up size hints.
                1..4 => {
                    for item in &self[..N - 1] {
                        sizes +=
                            simple_some!(size_hint_padded_versioned::<E, T, SIZE_BYTES, VERSION>(
                                item
                            ));
                    }
                    sizes += simple_some!(size_hint_versioned::<E, T, SIZE_BYTES, VERSION>(
                        &self[N - 1]
                    ));
                    Some(sizes)
                }
                _ => None,
            },
        }
    }
}

// Array of `N` elements of type `T` can be deserialized from a list of `N` elements of type `E` if `T` can be deserialized from `E`,
// and list formula is bounded to exactly `N` elements.
impl<'de, E, T, const N: usize, const VERSION: u32> Deserialize<'de, List<E, N, N>, VERSION>
    for [T; N]
where
    E: Element<VERSION> + ?Sized,
    T: Deserialize<'de, E::Formula, VERSION>,
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

        // for i in 0..N {
        //     let value = simple_try!(E::deserialize::<T, D>(&mut deserializer));
        //     options[i] = Some(value);
        // }

        let mut i = 0;
        simple_try!(E::deserialize_slice(&mut deserializer, N, |item| {
            options[i] = Some(item);
            i += 1;
        }));

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

        // for i in 0..N {
        //     simple_try!(E::deserialize_in_place::<T, D>(
        //         &mut self[i],
        //         &mut deserializer
        //     ));
        // }

        let mut i = 0;
        simple_try!(E::deserialize_slice(&mut deserializer, N, |item| {
            self[i] = item;
            i += 1;
        }));

        Ok(())
    }
}

element_alias!(for[const VERSION; E: Element<VERSION>, const N: usize] [E; N] as List<E, N, N>);
