use crate::{
    element::{Element, Indirect, heap_size, stack_size},
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
            simple_try!(serializer.write_usize(self.len()));
            for item in self {
                simple_try!(E::serialize(item, &mut serializer));
            }
        }
        Ok(())
    }

    #[inline]
    fn size_hint<const SIZE_BYTES: usize>(&self) -> Option<Sizes> {
        debug_assert!(self.is_empty() || E::INHABITED);

        if !E::INHABITED {
            // For uninhabited element type, slice can only be empty
            return Some(Sizes::ZERO);
        }

        let mut sizes = Sizes::with_stack(SIZE_BYTES);

        if self.is_empty() {
            return Some(sizes);
        }

        match (stack_size::<E, SIZE_BYTES>(), heap_size::<E, SIZE_BYTES>()) {
            (SizeBound::Bounded(max_stack), SizeBound::Exact(heap_size)) => {
                sizes.add_stack((self.len() - 1) * max_stack);
                sizes.add_heap(self.len() * heap_size);
                sizes.stack += simple_some!(self.last().unwrap().size_hint::<SIZE_BYTES>()).stack;
                Some(sizes)
            }
            (SizeBound::Exact(max_stack), SizeBound::Exact(heap_size)) => {
                sizes.add_stack(self.len() * max_stack);
                sizes.add_heap(self.len() * heap_size);
                Some(sizes)
            }
            _ => match self.len() {
                // For short slices, just sum up size hints.
                0..4 => {
                    for item in self {
                        sizes += simple_some!(E::size_hint::<T, SIZE_BYTES>(item));
                    }
                    Some(sizes)
                }
                _ => None,
            },
        }
    }
}

formula_alias!(for[E: Element] [E] as Indirect<List<E>>);
