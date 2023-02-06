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
    deserialize::{deserialize, deserialize_in_place, Deserialize, Error},
    formula::Formula,
    reference::Ref,
    serialize::{serialize, serialize_or_size, serialized_size, Serialize, SerializeOwned},
    size::{FixedIsize, FixedUsize},
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
        formula::{combine_sizes, Formula, NonRefFormula},
        serialize::{Serialize, SerializeOwned, Serializer},
    };

    pub struct WithFormula<F: Formula + ?Sized> {
        marker: PhantomData<fn(&F) -> &F>,
    }

    impl<F> WithFormula<F>
    where
        F: Formula + ?Sized,
    {
        #[cfg_attr(feature = "inline-more", inline(always))]
        pub fn write_value<T, S>(self, ser: &mut S, value: T) -> Result<(), S::Error>
        where
            S: Serializer,
            T: SerializeOwned<F::NonRef>,
        {
            ser.write_value::<F, T>(value)
        }

        #[cfg_attr(feature = "inline-more", inline(always))]
        pub fn read_value<'de, T>(self, des: &mut Deserializer<'de>) -> Result<T, Error>
        where
            F: Formula,
            T: Deserialize<'de, F::NonRef>,
        {
            des.read_value::<F, T>()
        }

        #[cfg_attr(feature = "inline-more", inline(always))]
        pub fn read_in_place<'de, T>(
            self,
            place: &mut T,
            des: &mut Deserializer<'de>,
        ) -> Result<(), Error>
        where
            F: Formula,
            T: Deserialize<'de, F::NonRef>,
        {
            des.read_in_place::<F, T>(place)
        }
    }

    #[cfg_attr(feature = "inline-more", inline(always))]
    pub fn with_formula<F: Formula + ?Sized, L: Formula + ?Sized>(
        _: impl FnOnce(&F) -> &L,
    ) -> WithFormula<L> {
        WithFormula {
            marker: PhantomData,
        }
    }
}
