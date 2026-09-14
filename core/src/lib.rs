#![cfg_attr(not(feature = "std"), no_std)]
// #![forbid(unsafe_code)]

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
    (for[$($generic:tt)*] $alias:ty as $element:ty) => {
        impl < $($generic)* > $crate::Element for $alias {
            $crate::formula_alias!(@impl $alias as $element);
        }
    };
    ($alias:ty as $element:ty) => {
        impl $crate::Element for $alias {
            $crate::formula_alias!(@impl $alias as $element);
        }
    };
    (@impl $alias:ty as $element:ty) => {
        type Formula = <$element as $crate::Element>::Formula;

        type StackSize<const SIZE_BYTES: usize> = <$element as $crate::Element>::StackSize<SIZE_BYTES>;
        type HeapSize<const SIZE_BYTES: usize> = <$element as $crate::Element>::HeapSize<SIZE_BYTES>;
        const INHABITED: bool = <$element as $crate::Element>::INHABITED;

        #[inline]
        fn serialize<T, S>(value: &T, serializer: &mut S) -> Result<(), S::Error>
        where
            T: $crate::Serialize<<$element as $crate::Element>::Formula> + ?Sized,
            S: $crate::Serializer,
        {
            <$element as $crate::Element>::serialize::<T, S>(value, serializer)
        }

        #[inline]
        fn size_hint<T, const SIZE_BYTES: usize>(value: &T) -> Option<$crate::Sizes>
        where
            T: $crate::serialize::Serialize<<$element as $crate::Element>::Formula> + ?Sized,
        {
            <$element as $crate::Element>::size_hint::<T, SIZE_BYTES>(value)
        }

        #[inline]
        fn deserialize<'de, T, D>(deserializer: &mut D) -> Result<T, $crate::DeserializeError>
        where
            T: $crate::Deserialize<'de, <$element as $crate::Element>::Formula>,
            D: $crate::Deserializer<'de>,
        {
            <$element as $crate::Element>::deserialize::<T, D>(deserializer)
        }

        #[inline]
        fn deserialize_in_place<'de, T, D>(
            place: &mut T,
            deserializer: &mut D,
        ) -> Result<(), $crate::DeserializeError>
        where
            T: $crate::Deserialize<'de, <$element as $crate::Element>::Formula> + ?Sized,
            D: $crate::Deserializer<'de>,
        {
            <$element as $crate::Element>::deserialize_in_place::<T, D>(place, deserializer)
        }
    };
}

/// Turn one of valid size_bytes value (valid values are in range `1..=16`)
/// into a const parameter for the given expression.
macro_rules! with_size_bytes {
    ($bind_size_bytes:ident = $size_bytes:expr => { $expr:expr } else $err:block) => {
        {
            match $size_bytes {
                1 => {
                    const $bind_size_bytes: usize = 1;
                    $expr
                }
                2 => {
                    const $bind_size_bytes: usize = 2;
                    $expr
                }
                3 => {
                    const $bind_size_bytes: usize = 3;
                    $expr
                }
                4 => {
                    const $bind_size_bytes: usize = 4;
                    $expr
                }
                5 => {
                    const $bind_size_bytes: usize = 5;
                    $expr
                }
                6 => {
                    const $bind_size_bytes: usize = 6;
                    $expr
                }
                7 => {
                    const $bind_size_bytes: usize = 7;
                    $expr
                }
                8 => {
                    const $bind_size_bytes: usize = 8;
                    $expr
                }
                9 => {
                    const $bind_size_bytes: usize = 9;
                    $expr
                }
                10 => {
                    const $bind_size_bytes: usize = 10;
                    $expr
                }
                11 => {
                    const $bind_size_bytes: usize = 11;
                    $expr
                }
                12 => {
                    const $bind_size_bytes: usize = 12;
                    $expr
                }
                13 => {
                    const $bind_size_bytes: usize = 13;
                    $expr
                }
                14 => {
                    const $bind_size_bytes: usize = 14;
                    $expr
                }
                15 => {
                    const $bind_size_bytes: usize = 15;
                    $expr
                }
                16 => {
                    const $bind_size_bytes: usize = 16;
                    $expr
                }
                _ => $err,
            }
        }
    };
    ($size_bytes:ident => { $expr:expr } else { $err:expr }) => {
        with_size_bytes!($size_bytes = $size_bytes => { $expr } else { $err })
    };
}

// Macro to avoid `?` operator for `Result`.
macro_rules! simple_try {
    ($x:expr) => {{
        match $x {
            Ok(value) => value,
            Err(err) => return Err(err),
        }
    }};
}

// Macro to avoid `?` operator for `Option`.
macro_rules! simple_some {
    ($x:expr) => {{
        match $x {
            Some(value) => value,
            None => return None,
        }
    }};
}

mod array;
mod buffer;
mod deserialize;
mod element;
mod formula;
mod iter;
mod lazy;
mod list;
mod never;
mod option;
mod packet;
mod primitive;
mod serialize;
mod slice;
mod str;
mod tuple;

#[cfg(feature = "alloc")]
mod string;

#[cfg(feature = "alloc")]
mod vec;

pub use self::{
    deserialize::{Deserialize, DeserializeError, Deserializer},
    element::{Element, Indirect, heap_size, inhabited, stack_size},
    formula::{BoundedSize, ExactSize, Formula, SizeBound, SizeType, UnboundedSize},
    iter::MakeIter,
    lazy::Lazy,
    list::{Array, List},
    never::Never,
    serialize::{Serialize, Serializer, Sizes},
    str::Str,
};

/// Module containing advanced facilities.
///
/// For example to manually implement [`Buffer`] and turn it into a [`Serializer`] using [`make_serializer`].
///
/// [`Buffer`]: crate::buffer::Buffer
/// [`Serializer`]: crate::serialize::Serializer
/// [`make_serializer`]: crate::serialize::make_serializer
pub mod advanced {
    pub use crate::{
        buffer::{
            Buffer, BufferExhausted, CheckedFixedBuffer, DryBuffer, MaybeFixedBuffer, VecBuffer,
        },
        packet::pack_into,
        serialize::{make_serializer, serialize_into, size_hint, write_usize},
    };
}

/// A trait that combines Formula, Serialize<Self> and Deserialize<Self>.
/// Automatically implemented for all types that implement the required traits.
pub trait Mixture: Formula + Serialize<Self> + for<'de> Deserialize<'de, Self> {}
impl<T> Mixture for T where T: Formula + Serialize<Self> + for<'de> Deserialize<'de, Self> {}

/// A trait that combines Element, Serialize<Self> and Deserialize<Self>.
/// Automatically implemented for all types that implement the required traits.
pub trait Component:
    Element + Serialize<Self::Formula> + for<'de> Deserialize<'de, Self::Formula>
{
}

impl<T> Component for T where
    T: Element + Serialize<Self::Formula> + for<'de> Deserialize<'de, Self::Formula>
{
}

/// Module containing functions for serialization/deserialization with manually specified size in bytes for length and address fields.
pub mod manual_size {
    pub use crate::{
        deserialize::{deserialize, deserialize_in_place},
        packet::{
            pack, pack_or_size, pack_size, pack_to_vec, pack_unchecked, read_pack_size, unpack,
            unpack_in_place,
        },
        serialize::{serialize, serialize_or_size, serialize_unchecked, serialized_size},
    };

    #[cfg(feature = "alloc")]
    pub use crate::serialize::serialize_to_vec;
}

/// Module containing functions for serialization/deserialization that use
/// 1 byte for length and address fields, suitable for small data (up to 255B / 255 elements).
///
/// Common use case - short messages with bounded small footprint such as commands transmitted over network.
pub mod small {
    pub use crate::{
        deserialize::small::{deserialize, deserialize_in_place},
        packet::small::{
            pack, pack_or_size, pack_size, pack_to_vec, pack_unchecked, read_pack_size, unpack,
            unpack_in_place,
        },
        serialize::small::{serialize, serialize_or_size, serialize_unchecked, serialized_size},
    };

    #[cfg(feature = "alloc")]
    pub use crate::serialize::small::serialize_to_vec;
}

/// Module containing functions for serialization/deserialization that use
/// 2 bytes for length and address fields, suitable for medium data (up to 65KB / 65K elements).
///
/// Common use case - normal-sized packets transmitted over network.
pub mod medium {
    pub use crate::{
        deserialize::medium::{deserialize, deserialize_in_place},
        packet::medium::{
            pack, pack_or_size, pack_size, pack_to_vec, pack_unchecked, read_pack_size, unpack,
            unpack_in_place,
        },
        serialize::medium::{serialize, serialize_or_size, serialize_unchecked, serialized_size},
    };

    #[cfg(feature = "alloc")]
    pub use crate::serialize::medium::serialize_to_vec;
}

/// Module containing functions for serialization/deserialization that use
/// 4 bytes for length and address fields, suitable for large data (up to 4GB / 4B elements).
///
/// Common use case - data stored in files.
pub mod large {
    pub use crate::{
        deserialize::large::{deserialize, deserialize_in_place},
        packet::large::{
            pack, pack_or_size, pack_size, pack_to_vec, pack_unchecked, read_pack_size, unpack,
            unpack_in_place,
        },
        serialize::large::{serialize, serialize_or_size, serialize_unchecked, serialized_size},
    };

    #[cfg(feature = "alloc")]
    pub use crate::serialize::large::serialize_to_vec;
}

/// Module containing functions for serialization/deserialization that use
/// 8 bytes for length and address fields, suitable for huge data (up to 16EB / 18 quintillion elements).
///
/// Common use case - data stored in large files exceeding 4GB.
pub mod huge {
    pub use crate::{
        deserialize::huge::{deserialize, deserialize_in_place},
        packet::huge::{
            pack, pack_or_size, pack_size, pack_to_vec, pack_unchecked, read_pack_size, unpack,
            unpack_in_place,
        },
        serialize::huge::{serialize, serialize_or_size, serialize_unchecked, serialized_size},
    };

    #[cfg(feature = "alloc")]
    pub use crate::serialize::huge::serialize_to_vec;
}

/// Module containing functions for serialization/deserialization that use
/// 16 bytes for length and address fields, suitable for humongous data (up to 256 UB / 340 undecillion elements).
///
/// Common use case - on today data scale no single message would overflow 16EB of [`huge`] category,
/// but why not prepare for the future when 17EB of data may be transmitted in a single message, or stored in a single file?
pub mod humongous {
    pub use crate::{
        deserialize::humongous::{deserialize, deserialize_in_place},
        packet::humongous::{
            pack, pack_or_size, pack_size, pack_to_vec, pack_unchecked, read_pack_size, unpack,
            unpack_in_place,
        },
        serialize::humongous::{
            serialize, serialize_or_size, serialize_unchecked, serialized_size,
        },
    };

    #[cfg(feature = "alloc")]
    pub use crate::serialize::humongous::serialize_to_vec;
}

// Re-exports large size functions as default for convenience.
// Large size is chosen as 16KB is commonly exceeded, so medium would fail.
// 4 byte per address and length would not incur too much overhead for small values.
pub use self::large::*;

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

    pub use {bool, f32, f64, i8, i16, i32, i64, i128, u8, u16, u32, u64, u128};

    #[inline]
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
        #[inline]
        pub fn serialize<T, S>(self, value: &T, serializer: &mut S) -> Result<(), S::Error>
        where
            T: Serialize<E::Formula> + ?Sized,
            S: Serializer,
        {
            E::serialize(value, serializer)
        }

        /// Helper function to take element formula from a composite formula.
        /// And then use it to get size hint for element direct serialization.
        #[inline]
        pub fn size_hint<T, const SIZE_BYTES: usize>(self, value: &T) -> Option<Sizes>
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

        #[inline]
        pub fn deserialize<'de, T, D>(self, deserializer: &mut D) -> Result<T, DeserializeError>
        where
            T: Deserialize<'de, E::Formula>,
            D: Deserializer<'de>,
        {
            E::deserialize(deserializer)
        }

        #[inline]
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

    #[inline]
    pub const fn discriminant_size(count: usize) -> usize {
        match count {
            0..=0xFF => 1,
            0x100..=0xFFFF => 2,
            0x10000..=0xFFFFFF => 3,
            0x10000..=0xFFFFFFFF => 4,
            _ => panic!("Too many enum variants"),
        }
    }

    /// Helper function to serialize enum discriminant.
    #[inline]
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
    #[inline]
    pub fn deserialize_discriminant<'de, D>(
        count: usize,
        deserializer: &mut D,
    ) -> Result<usize, DeserializeError>
    where
        D: Deserializer<'de>,
    {
        let size = discriminant_size(count);
        let bytes: &[u8] = simple_try!(deserializer.read_bytes(size));
        let mut array = [0u8; 4];

        array[..size].copy_from_slice(bytes);
        let idx = u32::from_le_bytes(array);

        match usize::try_from(idx) {
            Ok(idx) if idx < count => Ok(idx),
            _ => Err(DeserializeError::TooLarge(u128::from(idx))),
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
