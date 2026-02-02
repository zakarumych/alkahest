use crate::{
    element::{Element, heap_size, stack_size},
    formula::SizeBound,
    list::Array,
    serialize::{Serialize, Serializer, Sizes},
};

impl<E, T, const N: usize> Serialize<Array<E, N>> for [T; N]
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
            assert!(N == 0 || E::INHABITED);
        }

        for item in self {
            E::serialize(item, &mut serializer)?;
        }
        Ok(())
    }

    #[inline(always)]
    fn size_hint<const SIZE_BYTES: u8>(&self) -> Option<Sizes> {
        const {
            assert!(N == 0 || E::INHABITED);
        }

        let mut sizes = Sizes::ZERO;

        if N == 0 {
            return Some(sizes);
        }

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
