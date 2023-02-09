#![no_std]
#![deny(unsafe_code)]

extern crate self as alkahest;

#[cfg(feature = "alloc")]
extern crate alloc;

mod array;
mod bytes;
mod cold;
mod deserialize;
mod formula;
mod iter;
mod lazy;
mod option;
mod primitive;
mod reference;
mod serialize;
mod size;
mod skip;
mod slice;
mod str;
mod tuple;

#[cfg(feature = "alloc")]
mod vec;

#[cfg(feature = "alloc")]
mod vec_deque;

#[cfg(feature = "alloc")]
mod string;

pub use crate::{
    bytes::Bytes,
    deserialize::{deserialize, deserialize_in_place, value_size, DeIter, Deserialize, Error},
    formula::Formula,
    lazy::Lazy,
    reference::Ref,
    serialize::{serialize, serialize_or_size, serialized_size, Serialize, Serializer},
    size::{FixedIsize, FixedUsize},
    skip::Skip,
    slice::{LazySlice, SerIter},
};

#[cfg(feature = "derive")]
pub use alkahest_proc::{Deserialize, Formula, Serialize};

/// Private module for macros to use.
#[cfg(feature = "derive")]
#[doc(hidden)]
pub mod private {
    pub use {bool, u32, u8, usize, Into, Option, Result};

    pub use crate::{
        cold::{cold, err},
        deserialize::{Deserialize, Deserializer, Error},
        formula::{max_size, sum_size, Formula, NonRefFormula},
        serialize::{Serialize, Serializer},
    };

    use core::marker::PhantomData;

    pub const VARIANT_SIZE: usize = core::mem::size_of::<u32>();
    pub const VARIANT_SIZE_OPT: Option<usize> = Some(VARIANT_SIZE);

    pub struct WithFormula<F: Formula + ?Sized> {
        marker: PhantomData<fn(&F) -> &F>,
    }

    impl<F> WithFormula<F>
    where
        F: Formula + ?Sized,
    {
        #[inline(always)]
        pub fn write_value<T, S>(self, ser: &mut S, value: T) -> Result<(), S::Error>
        where
            S: Serializer,
            T: Serialize<F>,
        {
            ser.write_value::<F, T>(value)
        }

        #[inline(always)]
        pub fn read_value<'de, T>(self, de: &mut Deserializer<'de>) -> Result<T, Error>
        where
            F: Formula,
            T: Deserialize<'de, F>,
        {
            <T as Deserialize<F>>::deserialize(de.sub::<F>())
        }

        #[inline(always)]
        pub fn read_in_place<'de, T>(
            self,
            place: &mut T,
            de: &mut Deserializer<'de>,
        ) -> Result<(), Error>
        where
            F: Formula,
            T: Deserialize<'de, F>,
        {
            <T as Deserialize<F>>::deserialize_in_place(place, de.sub::<F>())
        }

        #[inline(always)]
        pub fn fast_sizes<T>(self, value: &T) -> Option<usize>
        where
            T: Serialize<F>,
        {
            <T as Serialize<F>>::fast_sizes(value)
        }
    }

    #[inline(always)]
    pub fn with_formula<F: Formula + ?Sized, L: Formula + ?Sized>(
        _: impl FnOnce(&F) -> &L,
    ) -> WithFormula<L> {
        WithFormula {
            marker: PhantomData,
        }
    }

    #[inline(always)]
    pub fn formula_fast_sizes<F>() -> Option<usize>
    where
        F: Formula + ?Sized,
    {
        match (F::EXACT_SIZE, F::HEAPLESS, F::MAX_STACK_SIZE) {
            (true, true, Some(max_stack_size)) => Some(max_stack_size),
            _ => None,
        }
    }
}
