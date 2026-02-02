use crate::{
    element::{Element, heap_size, stack_size},
    formula::SizeBound,
    list::List,
    serialize::{Serialize, Serializer, Sizes},
};

impl<E, T> Serialize<List<E>> for [T]
where
    E: Element,
    T: Serialize<E::Formula>,
{
    #[inline]
    fn serialize<S>(&self, mut serializer: S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        debug_assert!(self.is_empty() || E::INHABITED);

        if E::INHABITED {
            serializer.write_usize(self.len())?;
            for item in self {
                E::serialize(item, &mut serializer)?;
            }
        }
        Ok(())
    }

    #[inline(always)]
    fn size_hint<const SIZE_BYTES: u8>(&self) -> Option<Sizes> {
        debug_assert!(self.is_empty() || E::INHABITED);

        if !E::INHABITED {
            // For uninhabited element type, slice can only be empty
            return Some(Sizes::ZERO);
        }

        let mut sizes = Sizes::with_stack(usize::from(SIZE_BYTES));

        if self.is_empty() {
            return Some(sizes);
        }

        match (stack_size::<E, SIZE_BYTES>(), heap_size::<E, SIZE_BYTES>()) {
            (SizeBound::Bounded(max_stack), SizeBound::Exact(heap_size)) => {
                // For heapless types, we can optimize size hint calculation
                // by using max stack size for all but the last element.
                // and adding size hint for the last element.

                sizes.add_stack((self.len() - 1) * max_stack);
                sizes += self.last().unwrap().size_hint::<SIZE_BYTES>()?;
                sizes.add_heap(self.len() * heap_size);
                Some(sizes)
            }
            (SizeBound::Exact(max_stack), SizeBound::Exact(heap_size)) => {
                // For heapless types, we can optimize size hint calculation
                // by using max stack size for all but the last element.
                // and adding size hint for the last element.

                sizes.add_stack(self.len() * max_stack);
                sizes.add_heap(self.len() * heap_size);
                Some(sizes)
            }
            _ => match self.len() {
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
