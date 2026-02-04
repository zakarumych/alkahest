#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[allow(unused_macros)]
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

#[allow(unused_macros)]
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

#[macro_export]
macro_rules! formula_alias {
    ($(for[$($generic:tt),*])? $alias:ty as $formula:ty) => {
        impl $(< $($generic),* >)? $crate::Element for $alias {
            type Formula = $formula;

            type StackSize<const SIZE_BYTES: u8> = <$formula as $crate::Formula>::StackSize<SIZE_BYTES>;
            type HeapSize<const SIZE_BYTES: u8> = <$formula as $crate::Formula>::HeapSize<SIZE_BYTES>;
            const INHABITED: bool = true;

            fn serialize<T, S>(value: &T, serializer: &mut S) -> Result<(), S::Error>
            where
                T: $crate::Serialize<$formula> + ?Sized,
                S: $crate::Serializer,
            {
                serializer.write_direct(value)
            }

            fn size_hint<T, const SIZE_BYTES: u8>(value: &T) -> Option<$crate::Sizes>
            where
                T: $crate::serialize::Serialize<$formula> + ?Sized,
            {
                value.size_hint::<SIZE_BYTES>()
            }

            fn deserialize<'de, T, D>(deserializer: &mut D) -> Result<T, $crate::DeserializeError>
            where
                T: $crate::Deserialize<'de, $formula>,
                D: $crate::Deserializer<'de>,
            {
                deserializer.read_direct()
            }

            fn deserialize_in_place<'de, T, D>(
                place: &mut T,
                deserializer: &mut D,
            ) -> Result<(), $crate::DeserializeError>
            where
                T: $crate::Deserialize<'de, $formula> + ?Sized,
                D: $crate::Deserializer<'de>,
            {
                deserializer.read_direct_in_place(place)
            }
        }
    };
}

mod array;
mod buffer;
mod deserialize;
mod element;
mod formula;
// mod iter;
mod list;
mod never;
mod option;
mod primitive;
mod serialize;
mod slice;
mod str;
mod string;
mod tuple;

#[cfg(feature = "alloc")]
mod vec;

pub use self::{
    deserialize::{Deserialize, DeserializeError, Deserializer, deserialize, deserialize_in_place},
    element::{Element, Indirect, heap_size, inhabited, stack_size},
    formula::{BoundedSize, ExactSize, Formula, SizeBound, SizeType, UnboundedSize},
    list::{Array, List},
    never::Never,
    serialize::{
        Serialize, Serializer, Sizes, serialize, serialize_into, serialize_or_size,
        serialize_unchecked, serialized_sizes, size_hint,
    },
    string::String,
};

#[cfg(feature = "alloc")]
pub use self::serialize::serialize_to_vec;

/// A trait that combines Formula, Serialize<Self> and Deserialize<Self>.
/// Automatically implemented for all types that implement the required traits.
pub trait Mixture: Formula + Serialize<Self> + for<'de> Deserialize<'de, Self> {}
impl<T> Mixture for T where T: Formula + Serialize<Self> + for<'de> Deserialize<'de, Self> {}

/// A trait that combines Element, Serialize<Self> and Deserialize<Self>.
/// Automatically implemented for all types that implement the required traits.
pub trait MixtureElement:
    Element + Serialize<Self::Formula> + for<'de> Deserialize<'de, Self::Formula>
{
}

impl<T> MixtureElement for T where
    T: Element + Serialize<Self::Formula> + for<'de> Deserialize<'de, Self::Formula>
{
}

/// Module containing facilities for macro-generated code.
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
pub mod private {
    use crate::{
        Deserialize, Deserializer,
        deserialize::DeserializeError,
        element::{Element, heap_size, stack_size},
        formula::{Formula, SizeBound},
        serialize::{Serialize, Serializer, Sizes},
    };

    pub use core::{
        marker::PhantomData,
        option::Option::{self, None, Some},
        result::Result::{self, Err, Ok},
    };

    pub use {bool, f32, f64, i8, i16, i32, i64, i128, str, u8, u16, u32, u64, u128};

    #[inline(always)]
    pub fn with_element<F, E>(f: impl FnOnce(&F) -> &E) -> WithElement<E>
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
            T: Serialize<E::Formula> + ?Sized,
            S: Serializer,
        {
            E::serialize(value, serializer)
        }

        /// Helper function to take element formula from a composite formula.
        /// And then use it to get size hint for element direct serialization.
        #[inline(always)]
        pub fn size_hint<T, const SIZE_BYTES: u8>(self, value: &T) -> Option<Sizes>
        where
            T: Serialize<E::Formula> + ?Sized,
        {
            match const { (stack_size::<E, SIZE_BYTES>(), heap_size::<E, SIZE_BYTES>()) } {
                (SizeBound::Exact(stack), SizeBound::Exact(heap)) => {
                    return Some(Sizes { stack, heap });
                }
                _ => E::size_hint::<T, SIZE_BYTES>(value),
            }
        }

        #[inline(always)]
        pub fn deserialize<'de, T, D>(self, deserializer: &mut D) -> Result<T, DeserializeError>
        where
            T: Deserialize<'de, E::Formula>,
            D: Deserializer<'de>,
        {
            E::deserialize(deserializer)
        }

        #[inline(always)]
        pub fn deserialize_in_place<'de, T, D>(
            self,
            place: &mut T,
            deserializer: &mut D,
        ) -> Result<(), DeserializeError>
        where
            T: Deserialize<'de, E::Formula> + ?Sized,
            D: Deserializer<'de>,
        {
            E::deserialize_in_place(place, deserializer)
        }
    }

    #[inline(always)]
    pub const fn discriminant_size(count: usize) -> usize {
        match count {
            0..=0xFF => 1,
            0x100..=0xFFFF => 2,
            0x10000..=0xFFFFFFFF => 4,
            _ => panic!("Too many enum variants"),
        }
    }

    /// Helper function to serialize enum discriminant.
    #[inline(always)]
    pub fn serialize_discriminant<S>(
        idx: usize,
        count: usize,
        serializer: &mut S,
    ) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        let size = discriminant_size(count);
        let bytes = idx.to_le_bytes();
        assert!(size <= bytes.len());
        debug_assert!(bytes[size..].iter().all(|&b| b == 0));
        serializer.write_bytes(&bytes[..size])
    }

    /// Helper function to deserialize enum discriminant.
    #[inline(always)]
    pub fn deserialize_discriminant<'de, D>(
        count: usize,
        deserializer: &mut D,
    ) -> Result<usize, DeserializeError>
    where
        D: Deserializer<'de>,
    {
        let size = discriminant_size(count);
        let bytes: &[u8] = deserializer.read_bytes(size)?;
        let mut array = [0u8; 4];

        array[..size].copy_from_slice(bytes);
        let idx = u32::from_le_bytes(array);

        match usize::try_from(idx) {
            Ok(idx) if idx < count => Ok(idx),
            _ => Err(DeserializeError::InvalidUsize(u128::from(idx))),
        }
    }

    pub trait DeserializeEnumVariant<'de, F: Formula + ?Sized> {
        fn deserialize_enum_variant<D>(
            discriminant: usize,
            deserializer: D,
        ) -> Result<Self, DeserializeError>
        where
            D: Deserializer<'de>,
            Self: Sized;
    }
}
