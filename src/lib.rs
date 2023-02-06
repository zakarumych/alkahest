#![no_std]
#![deny(unsafe_code)]

extern crate self as alkahest;

#[cfg(feature = "alloc")]
extern crate alloc;

mod array;
mod bytes;
mod deserialize;
mod formula;
mod lazy;
mod option;
mod primitive;
mod reference;
mod serialize;
mod size;
mod skip;
mod slice;
mod tuple;

#[cfg(feature = "alloc")]
mod vec;

#[cfg(feature = "alloc")]
mod vec_deque;

pub use crate::{
    bytes::Bytes,
    deserialize::{deserialize, deserialize_in_place, value_size, Deserialize, Error},
    formula::Formula,
    lazy::Lazy,
    reference::Ref,
    serialize::{serialize, serialize_or_size, serialized_size, Serialize},
    size::{FixedIsize, FixedUsize},
    skip::Skip,
    slice::{SerIter, SliceIter},
};

#[cfg(feature = "derive")]
pub use alkahest_proc::{Deserialize, Formula, Serialize};

/// Private module for macros to use.
#[cfg(feature = "derive")]
#[doc(hidden)]
pub mod private {
    pub use {bool, u32, u8, usize, Into, Option, Result};

    use core::marker::PhantomData;

    pub use crate::{
        deserialize::{Deserialize, Deserializer, Error},
        formula::{max_size, sum_size, Formula, NonRefFormula},
        serialize::{Serialize, Serializer},
    };

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
        pub fn read_value<'de, T>(self, des: &mut Deserializer<'de>) -> Result<T, Error>
        where
            F: Formula,
            T: Deserialize<'de, F>,
        {
            des.read_value::<F, T>()
        }

        #[inline(always)]
        pub fn read_in_place<'de, T>(
            self,
            place: &mut T,
            des: &mut Deserializer<'de>,
        ) -> Result<(), Error>
        where
            F: Formula,
            T: Deserialize<'de, F>,
        {
            des.read_in_place::<F, T>(place)
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
}
