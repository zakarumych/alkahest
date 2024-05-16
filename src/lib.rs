#![doc = include_str!("../README.md")]
//!
//! The root module exports public API sufficient for most use cases.
//!
//! For manual implementation and direct usage of `Buffer`, `Formula`,
//! `Serialize` and `Deserialize` traits and `Deserializer` type
//! see [`advanced`] module.

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(
    clippy::correctness,
    clippy::suspicious,
    clippy::complexity,
    clippy::perf,
    clippy::style
)]

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
mod packet;
mod primitive;
mod reference;
mod serialize;
mod size;
mod skip;
mod slice;
mod str;
mod tuple;
mod vlq;

#[cfg(feature = "alloc")]
mod boxed;

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
    buffer::BufferExhausted,
    bytes::Bytes,
    deserialize::{
        deserialize, deserialize_in_place, deserialize_in_place_with_size, deserialize_with_size,
        DeIter, Deserialize, DeserializeError,
    },
    formula::Formula,
    iter::SerIter,
    lazy::Lazy,
    packet::{
        packet_size, read_packet, read_packet_in_place, read_packet_size, write_packet,
        write_packet_into, write_packet_unchecked,
    },
    r#as::As,
    reference::Ref,
    serialize::{
        serialize, serialize_or_size, serialize_unchecked, serialized_size, BufferSizeRequired,
        Serialize, SerializeRef,
    },
    skip::Skip,
    vlq::Vlq,
};

#[cfg(feature = "alloc")]
pub use crate::{packet::write_packet_to_vec, serialize::serialize_to_vec};

#[cfg(feature = "derive")]
pub use alkahest_proc::{alkahest, Deserialize, Formula, Serialize, SerializeRef};

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
            field_size_hint, formula_fast_sizes, slice_writer, write_array, write_bytes,
            write_exact_size_field, write_field, write_ref, write_reference, write_slice, Sizes,
            SliceWriter,
        },
        size::{FixedIsizeType, FixedUsizeType},
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
        serialize::{
            formula_fast_sizes, write_exact_size_field, write_field, Serialize, SerializeRef, Sizes,
        },
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

    #[must_use]
    #[inline(always)]
    pub fn with_formula<F: Formula + ?Sized, L: Formula + ?Sized>(
        _: impl FnOnce(&F) -> &L,
    ) -> WithFormula<L> {
        WithFormula {
            marker: PhantomData,
        }
    }
}
