use std::marker::PhantomData;

use crate::{
    element::{Element, heap_size, stack_size},
    formula::{Formula, SizeBound, SizeType},
};

/// List formula is a variable-length sequence of element formulas.
///
/// It has minimum and maximum length constraints, that can be 0 and usize::MAX to represent unbounded lists.
pub struct List<T: ?Sized, const MIN: usize = 0, const MAX: usize = { usize::MAX }>(PhantomData<T>);

/// Fixed-size array formula is a list with equal minimum and maximum sizes.
pub type Array<T, const N: usize> = List<T, N, N>;

pub struct ListStackSize<E: Element, const MIN: usize, const MAX: usize, const SIZE_BYTES: u8>(E);

impl<E, const MIN: usize, const MAX: usize, const SIZE_BYTES: u8> SizeType
    for ListStackSize<E, MIN, MAX, SIZE_BYTES>
where
    E: Element,
{
    const VALUE: SizeBound = if E::INHABITED {
        if MIN == MAX {
            // No need to store length if min == max
            match stack_size::<E, SIZE_BYTES>() {
                SizeBound::Unbounded => SizeBound::Unbounded,
                SizeBound::Bounded(size) => SizeBound::Bounded(size * MAX),
                SizeBound::Exact(size) => SizeBound::Exact(size * MAX),
            }
        } else {
            // Need to store length and size can't be exact
            match stack_size::<E, SIZE_BYTES>() {
                SizeBound::Unbounded => SizeBound::Unbounded,
                SizeBound::Bounded(size) => SizeBound::Bounded(size * MAX + SIZE_BYTES as usize),
                SizeBound::Exact(size) => SizeBound::Bounded(size * MAX + SIZE_BYTES as usize),
            }
        }
    } else {
        // if E is uninhabited, list can only be empty
        SizeBound::Exact(0)
    };
}

pub struct ListHeapSize<E: Element, const MIN: usize, const MAX: usize, const SIZE_BYTES: u8>(E);

impl<E, const MIN: usize, const MAX: usize, const SIZE_BYTES: u8> SizeType
    for ListHeapSize<E, MIN, MAX, SIZE_BYTES>
where
    E: Element,
{
    const VALUE: SizeBound = if E::INHABITED {
        if MIN == MAX {
            match heap_size::<E, SIZE_BYTES>() {
                SizeBound::Unbounded => SizeBound::Unbounded,
                SizeBound::Bounded(size) => SizeBound::Bounded(size * MAX),
                SizeBound::Exact(size) => SizeBound::Exact(size * MAX),
            }
        } else {
            // Size can't be exact
            match heap_size::<E, SIZE_BYTES>() {
                SizeBound::Unbounded => SizeBound::Unbounded,
                SizeBound::Bounded(size) => SizeBound::Bounded(size * MAX),
                SizeBound::Exact(size) => SizeBound::Bounded(size * MAX),
            }
        }
    } else {
        // if E is uninhabited, list can only be empty
        SizeBound::Exact(0)
    };
}

impl<E, const MIN: usize, const MAX: usize> Formula for List<E, MIN, MAX>
where
    E: Element,
{
    type StackSize<const SIZE_BYTES: u8> = ListStackSize<E, MIN, MAX, SIZE_BYTES>;
    type HeapSize<const SIZE_BYTES: u8> = ListHeapSize<E, MIN, MAX, SIZE_BYTES>;
    const INHABITED: bool = E::INHABITED || MIN == 0;
}
