use crate::{
    advanced::size_hint_versioned,
    element::{Element, Indirect, heap_size_versioned, stack_size_versioned},
    formula::SizeBound,
    list::List,
    serialize::{Serialize, Serializer, Sizes},
};

impl<E, T, const VERSION: u32> Serialize<List<E>, VERSION> for [T]
where
    E: Element<VERSION> + ?Sized,
    T: Serialize<E::Formula, VERSION>,
{
    #[inline]
    fn serialize<S>(&self, mut serializer: S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        debug_assert!(self.is_empty() || E::INHABITED);

        if !E::INHABITED {
            return Ok(());
        }

        simple_try!(serializer.write_usize(self.len()));
        // for item in self {
        //     simple_try!(E::serialize(item, &mut serializer));
        // }

        // Ok(())

        E::serialize_slice(self, &mut serializer)
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

        match const {
            (
                stack_size_versioned::<E, SIZE_BYTES, VERSION>(),
                heap_size_versioned::<E, SIZE_BYTES, VERSION>(),
            )
        } {
            (SizeBound::Bounded(max_stack), SizeBound::Exact(heap_size)) => {
                sizes.add_stack((self.len() - 1) * max_stack);
                sizes.add_heap(self.len() * heap_size);
                sizes.stack += simple_some!(size_hint_versioned::<E, T, SIZE_BYTES, VERSION>(
                    self.last().unwrap()
                ))
                .stack;
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
                    for (index, item) in self.iter().enumerate() {
                        let mut next =
                            simple_some!(size_hint_versioned::<E, T, SIZE_BYTES, VERSION>(item));
                        if index + 1 < self.len() {
                            if let SizeBound::Bounded(max_stack) =
                                stack_size_versioned::<E, SIZE_BYTES, VERSION>()
                            {
                                next.stack = max_stack;
                            }
                        }
                        sizes += next;
                    }
                    Some(sizes)
                }
                _ => None,
            },
        }
    }
}

element_alias!(for[const VERSION; E: Element<VERSION>] [E] as Indirect<List<E>>);
