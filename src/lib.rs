#![doc = include_str!("../README.md")]
//!
//! The root module exports public API sufficient for most use cases.
//! Except manual implementation and direct usage of `Buffer`, `Formula`,
//! `Serialize` and `Deserialize` traits and `Deserializer` type.
//! For those use cases, see `advanced` module.

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![deny(missing_docs)]

#[cfg(test)]
extern crate self as alkahest;

#[cfg(feature = "alloc")]
extern crate alloc;

mod array;
mod r#as;
mod buffer;
mod bytes;
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
mod vlq;

#[cfg(test)]
mod tests;

#[cfg(feature = "alloc")]
mod vec;

#[cfg(feature = "alloc")]
mod vec_deque;

#[cfg(feature = "alloc")]
mod string;

#[cfg(feature = "bincoded")]
mod bincoded;

pub use crate::{
    buffer::{BufferExhausted, BufferSizeRequired},
    bytes::Bytes,
    deserialize::{
        deserialize, deserialize_in_place, value_size, DeIter, Deserialize, DeserializeError,
    },
    formula::Formula,
    iter::SerIter,
    lazy::Lazy,
    r#as::As,
    reference::Ref,
    serialize::{serialize, serialize_or_size, serialize_unchecked, serialized_size, Serialize},
    size::{FixedIsize, FixedUsize},
    skip::Skip,
    vlq::Vlq,
};

#[cfg(feature = "alloc")]
pub use crate::serialize::serialize_to_vec;

#[cfg(feature = "derive")]
pub use alkahest_proc::{Deserialize, Formula, Serialize};

#[cfg(feature = "bincoded")]
pub use bincoded::{Bincode, Bincoded};

/// This module contains types and functions for manual implementations of
/// `Serialize` and `Deserialize` traits.
pub mod advanced {
    pub use crate::{
        buffer::{Buffer, CheckedFixedBuffer, MaybeFixedBuffer},
        deserialize::Deserializer,
        formula::{reference_size, BareFormula},
        iter::{default_iter_fast_sizes, deserialize_extend_iter, deserialize_from_iter},
        serialize::{
            field_size_hint, formula_fast_sizes, slice_writer, write_bytes, write_exact_size_field,
            write_field, write_ref, write_reference, write_slice, Sizes, SliceWriter,
        },
        size::{FixedIsize, FixedIsizeType},
    };

    #[cfg(feature = "alloc")]
    pub use crate::buffer::VecBuffer;
}

/// Private module for macros to use.
/// Changes here are not considered breaking.
#[doc(hidden)]
pub mod private {
    pub use {
        bool,
        core::{convert::Into, debug_assert_eq, option::Option, result::Result},
        u32, u8, usize,
    };

    pub use crate::{
        buffer::Buffer,
        deserialize::{Deserialize, DeserializeError, Deserializer},
        formula::{max_size, sum_size, BareFormula, Formula},
        serialize::{formula_fast_sizes, write_exact_size_field, write_field, Serialize, Sizes},
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
        pub fn write_field<T, B>(
            self,
            value: T,
            sizes: &mut Sizes,
            buffer: B,
            last: bool,
        ) -> Result<(), B::Error>
        where
            B: Buffer,
            T: Serialize<F>,
        {
            crate::serialize::write_field(value, sizes, buffer, last)
        }

        #[inline(always)]
        pub fn read_field<'de, T>(
            self,
            de: &mut Deserializer<'de>,
            last: bool,
        ) -> Result<T, DeserializeError>
        where
            F: Formula,
            T: Deserialize<'de, F>,
        {
            de.read_value::<F, T>(last)
        }

        #[inline(always)]
        pub fn read_in_place<'de, T>(
            self,
            place: &mut T,
            de: &mut Deserializer<'de>,
            last: bool,
        ) -> Result<(), DeserializeError>
        where
            F: Formula,
            T: Deserialize<'de, F>,
        {
            de.read_in_place::<F, T>(place, last)
        }

        #[inline(always)]
        pub fn size_hint<T>(self, value: &T, last: bool) -> Option<Sizes>
        where
            T: Serialize<F>,
        {
            crate::serialize::field_size_hint::<F>(value, last)
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
