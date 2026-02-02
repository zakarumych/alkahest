#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]

#[cfg(feature = "alloc")]
extern crate alloc;

macro_rules! for_tuple {
    ($macro:ident) => {
        for_tuple!($macro for A B C D E F G H I J K L M N O P);
    };
    ($macro:ident for) => {
        $macro!();
    };
    ($macro:ident for $head:ident $($tail:ident)*) => {
        for_tuple!($macro for $($tail)*);

        $macro!($head $($tail)*);
    };
}

macro_rules! for_tuple_2 {
    ($macro:ident) => {
        for_tuple_2!($macro for
            AA AB AC AD AE AF AG AH AI AJ AK AL AM AN AO AP,
            BA BB BC BD BE BF BG BH BI BJ BK BL BM BN BO BP
        );
    };
    ($macro:ident for ,) => {
        $macro!(,);
    };
    ($macro:ident for $a_head:ident $($a_tail:ident)*, $b_head:ident $($b_tail:ident)*) => {
        for_tuple_2!($macro for $($a_tail)*, $($b_tail)*);

        $macro!($a_head $($a_tail)*, $b_head $($b_tail)*);
    };
}

mod array;
mod buffer;
mod deserialize;
mod element;
mod formula;
// mod iter;
mod list;
mod option;
mod primitive;
mod serialize;
mod slice;
mod str;
mod tuple;
mod void;

#[cfg(feature = "alloc")]
mod vec;

/// Module containing facilities for macro-generated code.
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
pub mod private {
    use crate::{
        element::Element,
        formula::Formula,
        serialize::{Serialize, Serializer, Sizes},
    };

    pub use {
        crate::{
            element::{
                Element as __Alkahest_Element, Indirect as __Alkahest_Indirect,
                heap_size as __Alkahest_heap_size, inhabited as __Alkahest_inhabited,
                stack_size as __Alkahest_stack_size,
            },
            formula::{
                BoundedSize as __Alkahest_BoundedSize, ExactSize as __Alkahest_ExactSize,
                Formula as __Alkahest_Formula, SizeBound as __Alkahest_SizeBound,
                SizeType as __Alkahest_SizeType, UnboundedSize as __Alkahest_UnboundedSize,
            },
            list::{Array as __Alkahest_Array, List as __Alkahest_List},
            serialize::{
                Serialize as __Alkahest_Serialize, Serializer as __Alkahest_Serializer,
                Sizes as __Alkahest_Sizes,
            },
            void::Void::{self as __Alkahest_Void, self}, // unprefixed name can be referenced in source code, prefixed name avoids shadowing and is used in macro-generated code
        },
        core::{
            marker::PhantomData as __Alkahest_PhantomData,
            option::Option::{
                self as __Alkahest_Option, None as __Alkahest_None, Some as __Alkahest_Some,
            },
            result::Result as __Alkahest_Result,
        },
    };

    #[inline(always)]
    pub fn __Alkahest_with_element<F, E>(f: impl FnOnce(&F) -> &E) -> WithElement<E>
    where
        F: Formula + ?Sized,
        E: Element + ?Sized,
    {
        let _ = f;
        WithElement {
            _marker: core::marker::PhantomData,
        }
    }

    pub struct WithElement<E: ?Sized> {
        _marker: core::marker::PhantomData<E>,
    }

    impl<E> WithElement<E>
    where
        E: Element + ?Sized,
    {
        /// Helper function to take element formula from a composite formula.
        /// And then use it to serialize element either directly.
        #[inline(always)]
        pub fn serialize<T, S>(self, value: &T, serializer: &mut S) -> Result<(), S::Error>
        where
            T: Serialize<E::Formula>,
            S: Serializer,
        {
            E::serialize(value, serializer)
        }

        /// Helper function to take element formula from a composite formula.
        /// And then use it to get size hint for element direct serialization.
        #[inline(always)]
        pub fn size_hint<T, const SIZE_BYTES: u8>(self, value: &T) -> Option<Sizes>
        where
            T: Serialize<E::Formula>,
        {
            match const {
                (
                    __Alkahest_stack_size::<E, SIZE_BYTES>(),
                    __Alkahest_heap_size::<E, SIZE_BYTES>(),
                )
            } {
                (__Alkahest_SizeBound::Exact(stack), __Alkahest_SizeBound::Exact(heap)) => {
                    return Some(__Alkahest_Sizes { stack, heap });
                }
                _ => E::size_hint::<T, SIZE_BYTES>(value),
            }
        }
    }

    pub const fn __Alkahest_discriminant_size(count: usize) -> usize {
        match count {
            0..=0xFF => 1,
            0x100..=0xFFFF => 2,
            0x10000..=0xFFFFFFFF => 4,
            _ => panic!("Too many enum variants"),
        }
    }

    /// Helper function to serialize enum discriminant.
    #[inline(always)]
    pub fn __Alkahest_serialize_discriminant<S>(
        idx: usize,
        count: usize,
        serializer: &mut S,
    ) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        let size = __Alkahest_discriminant_size(count);
        let bytes = idx.to_le_bytes();
        debug_assert!(bytes[size..].iter().all(|&b| b == 0));
        serializer.write_bytes(&bytes[..size])
    }
}
