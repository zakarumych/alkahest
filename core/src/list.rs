use core::marker::PhantomData;

use crate::{
    element::{Element, heap_size_versioned, stack_size_versioned},
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
    E: Element<VERSION> + ?Sized,
    const MIN: usize,
    const MAX: usize,
    const SIZE_BYTES: usize,
    const VERSION: u32 = 0,
>(E);

impl<E, const MIN: usize, const MAX: usize, const SIZE_BYTES: usize, const VERSION: u32> SizeType
    for ListStackSize<E, MIN, MAX, SIZE_BYTES, VERSION>
where
    E: Element<VERSION> + ?Sized,
{
    const VALUE: SizeBound = if E::INHABITED {
        assert!(
            MIN <= MAX,
            "List minimum length must be less than or equal to maximum length",
        );

        if MIN == MAX {
            // No need to store length if min == max
            stack_size_versioned::<E, SIZE_BYTES, VERSION>()
                .padded()
                .mul(MAX)
        } else {
            // Need to store length and size can't be exact
            stack_size_versioned::<E, SIZE_BYTES, VERSION>()
                .padded()
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
    E: Element<VERSION> + ?Sized,
    const MIN: usize,
    const MAX: usize,
    const SIZE_BYTES: usize,
    const VERSION: u32 = 0,
>(E);

impl<E, const MIN: usize, const MAX: usize, const SIZE_BYTES: usize, const VERSION: u32> SizeType
    for ListHeapSize<E, MIN, MAX, SIZE_BYTES, VERSION>
where
    E: Element<VERSION> + ?Sized,
{
    const VALUE: SizeBound = if E::INHABITED {
        assert!(
            MIN <= MAX,
            "List minimum length must be less than or equal to maximum length",
        );

        if MIN == MAX {
            heap_size_versioned::<E, SIZE_BYTES, VERSION>().mul(MAX)
        } else {
            // Size can't be exact
            heap_size_versioned::<E, SIZE_BYTES, VERSION>()
                .mul(MAX)
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

impl<E, const MIN: usize, const MAX: usize, const VERSION: u32> Formula<VERSION>
    for List<E, MIN, MAX>
where
    E: Element<VERSION> + ?Sized,
{
    const SCHEMA: crate::version::Schema =
        crate::version::Schema::new::<Self, VERSION>(crate::version::SchemaKind::List {
            element: E::SCHEMA,
            min: MIN,
            max: MAX,
        });
    type StackSize<const SIZE_BYTES: usize> = ListStackSize<E, MIN, MAX, SIZE_BYTES, VERSION>;
    type HeapSize<const SIZE_BYTES: usize> = ListHeapSize<E, MIN, MAX, SIZE_BYTES, VERSION>;
    const INHABITED: bool = E::INHABITED || MIN == 0;
}
