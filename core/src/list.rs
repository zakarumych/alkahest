use std::marker::PhantomData;

use crate::{
    element::{Element, heap_size, stack_size},
    formula::{Formula, SizeBound, SizeType},
};

/// List formula is a variable-length sequence of element formulas.
///
/// It has minimum and maximum length constraints.
/// The maximum of `usize::MAX` is used to represent an unbounded list and handled separately without arithmetic overflow.
/// Default minimum length is 0 and default maximum length is `usize::MAX`,
/// so that `List<T>` is a variable-length list of `T` with no length constraints.
pub struct List<T: ?Sized, const MIN: usize = 0, const MAX: usize = { usize::MAX }>(PhantomData<T>);

/// Fixed-size array formula is a list with equal minimum and maximum sizes.
pub type Array<T, const N: usize> = List<T, N, N>;

/// Stack size type for list formula.
pub struct ListStackSize<
    E: Element + ?Sized,
    const MIN: usize,
    const MAX: usize,
    const SIZE_BYTES: usize,
>(E);

impl<E, const MIN: usize, const MAX: usize, const SIZE_BYTES: usize> SizeType
    for ListStackSize<E, MIN, MAX, SIZE_BYTES>
where
    E: Element + ?Sized,
{
    const VALUE: SizeBound = if E::INHABITED {
        assert!(
            MIN <= MAX,
            "List minimum length must be less than or equal to maximum length",
        );

        if MIN == MAX {
            // No need to store length if min == max
            stack_size::<E, SIZE_BYTES>().mul(MAX)
        } else {
            // Need to store length and size can't be exact
            stack_size::<E, SIZE_BYTES>()
                .mul(MAX)
                .add(SizeBound::Exact(SIZE_BYTES))
                .not_exact()
        }
    } else {
        // if E is uninhabited, list can only be empty
        assert!(
            MIN == 0,
            "List with uninhabited element must allow empty list",
        );
        SizeBound::Exact(0)
    };
}

pub struct ListHeapSize<
    E: Element + ?Sized,
    const MIN: usize,
    const MAX: usize,
    const SIZE_BYTES: usize,
>(E);

impl<E, const MIN: usize, const MAX: usize, const SIZE_BYTES: usize> SizeType
    for ListHeapSize<E, MIN, MAX, SIZE_BYTES>
where
    E: Element + ?Sized,
{
    const VALUE: SizeBound = if E::INHABITED {
        assert!(
            MIN <= MAX,
            "List minimum length must be less than or equal to maximum length",
        );

        if MIN == MAX {
            heap_size::<E, SIZE_BYTES>().mul(MAX)
        } else {
            // Size can't be exact
            heap_size::<E, SIZE_BYTES>().mul(MAX).not_exact()
        }
    } else {
        // if E is uninhabited, list can only be empty
        assert!(
            MIN == 0,
            "List with uninhabited element must allow empty list",
        );
        SizeBound::Exact(0)
    };
}

impl<E, const MIN: usize, const MAX: usize> Formula for List<E, MIN, MAX>
where
    E: Element + ?Sized,
{
    type StackSize<const SIZE_BYTES: usize> = ListStackSize<E, MIN, MAX, SIZE_BYTES>;
    type HeapSize<const SIZE_BYTES: usize> = ListHeapSize<E, MIN, MAX, SIZE_BYTES>;
    const INHABITED: bool = E::INHABITED || MIN == 0;
}
