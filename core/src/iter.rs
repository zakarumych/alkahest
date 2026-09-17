use crate::{
    Element,
    list::List,
    serialize::{Serialize, Serializer, Sizes},
};

pub struct MakeIter<F>(pub F);

impl<E, T, F, I> Serialize<List<E>> for MakeIter<F>
where
    E: Element + ?Sized,
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
        // Materializing iterator and iterating it is too slow for size_hint.

        // let mut iter = (self.0)();

        if !E::INHABITED {
            // debug_assert!(
            //     iter.next().is_none(),
            //     "For uninhabited element type, iterator can only be empty"
            // );

            // For uninhabited element type, slice can only be empty
            return Some(Sizes::ZERO);
        }

        // let mut sizes = Sizes::with_stack(SIZE_BYTES);

        // match iter.size_hint() {
        //     (lower, Some(upper)) if lower == upper => {
        //         // Exact size case.
        //         let len = lower;

        //         if len == 0 {
        //             return Some(sizes);
        //         }

        //         match const { (stack_size::<E, SIZE_BYTES>(), heap_size::<E, SIZE_BYTES>()) } {
        //             (SizeBound::Exact(max_stack), SizeBound::Exact(heap_size)) => {
        //                 sizes.add_stack(len * max_stack);
        //                 sizes.add_heap(len * heap_size);
        //                 return Some(sizes);
        //             }
        //             _ => {}
        //         }
        //     }
        //     _ => {}
        // }

        // let Some(mut item) = iter.next() else {
        //     return Some(sizes);
        // };

        // for _ in 0..4 {
        //     if let Some(next) = iter.next() {
        //         sizes += simple_some!(size_hint_padded::<E, T, SIZE_BYTES>(&item));
        //         item = next;
        //     } else {
        //         sizes += simple_some!(size_hint::<E, T, SIZE_BYTES>(&item));
        //         return Some(sizes);
        //     }
        // }

        None
    }
}
