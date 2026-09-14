use crate::{
    Element, SizeBound, heap_size,
    list::List,
    serialize::{Serialize, Serializer, Sizes},
    stack_size,
};

pub struct MakeIter<F>(pub F);

impl<E, T, F, I> Serialize<List<E>> for MakeIter<F>
where
    E: Element,
    T: Serialize<E::Formula>,
    F: Fn() -> I,
    I: Iterator<Item = T>,
{
    fn serialize<S>(&self, mut serializer: S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        let mut iter = (self.0)();

        if E::INHABITED {
            let len_addr = simple_try!(serializer.reserve_usize());
            let mut count = 0usize;
            for item in iter {
                simple_try!(E::serialize(&item, &mut serializer));
                count += 1;
            }
            serializer.write_reserved_usize(len_addr, count);
        } else {
            debug_assert!(
                iter.next().is_none(),
                "For uninhabited element type, iterator can only be empty"
            );
        }

        Ok(())
    }

    fn size_hint<const SIZE_BYTES: usize>(&self) -> Option<Sizes> {
        let mut iter = (self.0)();

        if !E::INHABITED {
            debug_assert!(
                iter.next().is_none(),
                "For uninhabited element type, iterator can only be empty"
            );

            // For uninhabited element type, slice can only be empty
            return Some(Sizes::ZERO);
        }

        match iter.size_hint() {
            (lower, Some(upper)) if lower == upper => {
                // Exact size case.

                let len = lower;

                let mut sizes = Sizes::with_stack(SIZE_BYTES);

                if len == 0 {
                    return Some(sizes);
                }

                match (stack_size::<E, SIZE_BYTES>(), heap_size::<E, SIZE_BYTES>()) {
                    (SizeBound::Bounded(max_stack), SizeBound::Exact(heap_size)) => {
                        sizes.add_stack((len - 1) * max_stack);
                        sizes.add_heap(len * heap_size);
                        sizes.stack +=
                            simple_some!(iter.last().unwrap().size_hint::<SIZE_BYTES>()).stack;
                        Some(sizes)
                    }
                    (SizeBound::Exact(max_stack), SizeBound::Exact(heap_size)) => {
                        sizes.add_stack(len * max_stack);
                        sizes.add_heap(len * heap_size);
                        Some(sizes)
                    }
                    _ => match len {
                        // For short slices, just sum up size hints.
                        0..4 => {
                            for item in iter.by_ref().take(4) {
                                sizes += simple_some!(E::size_hint::<T, SIZE_BYTES>(&item));
                            }
                            debug_assert!(
                                iter.next().is_none(),
                                "Iterator should be empty after taking 4 elements"
                            );
                            Some(sizes)
                        }
                        _ => None,
                    },
                }
            }
            (lower, _) if lower > 4 => None,
            (lower, Some(upper)) if lower > upper => None,
            _ => {
                let mut sizes = Sizes::with_stack(SIZE_BYTES);

                for item in iter.by_ref().take(4) {
                    sizes += simple_some!(E::size_hint::<T, SIZE_BYTES>(&item));
                }

                if iter.next().is_some() {
                    None
                } else {
                    Some(sizes)
                }
            }
        }
    }
}
