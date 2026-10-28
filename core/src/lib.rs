#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[inline]
#[cold]
#[doc(hidden)]
pub fn cold_err<T, E>(e: E) -> Result<T, E> {
    Err(e)
}

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
macro_rules! element_alias {
    (for[const $version:ident; $($generic:tt)*] $alias:ty as $element:ty) => {
        impl<const $version: u32, $($generic)*> $crate::Element<$version> for $alias {
            $crate::element_alias!(@impl [$version] $alias as $element);
        }
    };
    (for[$($generic:tt)*] $alias:ty as $element:ty) => {
        impl < $($generic)* > $crate::Element for $alias {
            $crate::element_alias!(@impl [0] $alias as $element);
        }
    };
    ($alias:ty as $element:ty) => {
        impl $crate::Element for $alias {
            $crate::element_alias!(@impl [0] $alias as $element);
        }
    };
    (@impl [$version:expr] $alias:ty as $element:ty) => {
        type Formula = <$element as $crate::Element<$version>>::Formula;

        type StackSize<const SIZE_BYTES: usize> = <$element as $crate::Element<$version>>::StackSize<SIZE_BYTES>;
        type HeapSize<const SIZE_BYTES: usize> = <$element as $crate::Element<$version>>::HeapSize<SIZE_BYTES>;
        const SCHEMA: $crate::ElementSchema = <$element as $crate::Element<$version>>::SCHEMA;

        const INHABITED: bool = <$element as $crate::Element<$version>>::INHABITED;

        #[inline(always)]
        fn serialize<T, S>(value: &T, serializer: &mut S) -> Result<(), S::Error>
        where
            T: $crate::Serialize<<$element as $crate::Element<$version>>::Formula, $version> + ?Sized,
            S: $crate::Serializer,
        {
            <$element as $crate::Element<$version>>::serialize::<T, S>(value, serializer)
        }

        #[inline(always)]
        fn serialize_slice<T, S>(slice: &[T], serializer: &mut S) -> Result<(), S::Error>
        where
            T: $crate::Serialize<<$element as $crate::Element<$version>>::Formula, $version>,
            S: $crate::Serializer,
        {
            <$element as $crate::Element<$version>>::serialize_slice::<T, S>(slice, serializer)
        }

        #[inline(always)]
        fn size_hint<T, const SIZE_BYTES: usize>(value: &T) -> Option<$crate::Sizes>
        where
            T: $crate::Serialize<<$element as $crate::Element<$version>>::Formula, $version> + ?Sized,
        {
            <$element as $crate::Element<$version>>::size_hint::<T, SIZE_BYTES>(value)
        }

        #[inline(always)]
        fn deserialize<'de, T, D>(deserializer: &mut D) -> Result<T, $crate::DeserializeError>
        where
            T: $crate::Deserialize<'de, <$element as $crate::Element<$version>>::Formula, $version>,
            D: $crate::Deserializer<'de>,
        {
            <$element as $crate::Element<$version>>::deserialize::<T, D>(deserializer)
        }

        #[inline(always)]
        fn deserialize_slice<'de, T, D>(deserializer: &mut D, len: usize, push: impl FnMut(T)) -> Result<(), $crate::DeserializeError>
        where
            T: $crate::Deserialize<'de, <$element as $crate::Element<$version>>::Formula, $version>,
            D: $crate::Deserializer<'de>,
        {
            <$element as $crate::Element<$version>>::deserialize_slice::<T, D>(deserializer, len, push)
        }

        #[inline(always)]
        fn deserialize_in_place<'de, T, D>(
            place: &mut T,
            deserializer: &mut D,
        ) -> Result<(), $crate::DeserializeError>
        where
            T: $crate::Deserialize<'de, <$element as $crate::Element<$version>>::Formula, $version> + ?Sized,
            D: $crate::Deserializer<'de>,
        {
            <$element as $crate::Element<$version>>::deserialize_in_place::<T, D>(place, deserializer)
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
#[macro_export]
macro_rules! simple_try {
    ($x:expr) => {{
        match $x {
            Ok(value) => value,
            Err(err) => return $crate::cold_err(err),
        }
    }};
}

// Macro to avoid `?` operator for `Option`.
#[macro_export]
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
pub mod version;
pub mod versioned;

#[cfg(feature = "alloc")]
mod string;

#[cfg(feature = "alloc")]
mod vec;

pub use self::{
    deserialize::{Deserialize, DeserializeError, Deserializer},
    element::{
        Element, Indirect, heap_size, heap_size_versioned, inhabited, inhabited_versioned,
        stack_size, stack_size_versioned,
    },
    formula::{AsFormula, BoundedSize, ExactSize, Formula, SizeBound, SizeType, UnboundedSize},
    iter::MakeIter,
    lazy::Lazy,
    list::{Array, List},
    never::Never,
    serialize::{Serialize, Serializer, Sizes},
    str::Str,
    version::*,
    versioned::VersionError,
};

/// Module containing advanced facilities.
///
/// For example to manually implement [`Buffer`] and turn it into a [`Serializer`] using [`make_serializer`].
///
/// [`Buffer`]: crate::buffer::Buffer
/// [`Serializer`]: crate::serialize::Serializer
/// [`make_serializer`]: crate::serialize::make_serializer
pub mod advanced {
    #[cfg(feature = "alloc")]
    pub use crate::buffer::VecBuffer;
    pub use crate::{
        buffer::{Buffer, BufferExhausted, CheckedFixedBuffer, DryBuffer, MaybeFixedBuffer},
        packet::{pack_into, pack_into_at_version},
        serialize::{
            exact_sizes, exact_sizes_versioned, serialize_into, serialize_into_at_version,
            size_hint, size_hint_padded, size_hint_padded_versioned, size_hint_versioned,
            write_usize,
        },
    };
}

/// A trait that combines Formula, Serialize<Self> and Deserialize<Self>.
/// Automatically implemented for all types that implement the required traits.
pub trait Mixture<const VERSION: u32 = 0>:
    Formula<VERSION> + Serialize<Self, VERSION> + for<'de> Deserialize<'de, Self, VERSION>
{
}
impl<T, const VERSION: u32> Mixture<VERSION> for T where
    T: Formula<VERSION> + Serialize<Self, VERSION> + for<'de> Deserialize<'de, Self, VERSION>
{
}

/// A trait combining an element with its version's serialization traits.
pub trait Component<const VERSION: u32 = 0>:
    Element<VERSION>
    + Serialize<Self::Formula, VERSION>
    + for<'de> Deserialize<'de, Self::Formula, VERSION>
{
}
impl<T, const VERSION: u32> Component<VERSION> for T where
    T: Element<VERSION>
        + Serialize<Self::Formula, VERSION>
        + for<'de> Deserialize<'de, Self::Formula, VERSION>
{
}

/// Module containing functions for serialization/deserialization with manually specified size in bytes for length and address fields.
pub mod manual_size {
    #[cfg(feature = "alloc")]
    pub use crate::packet::{pack_to_vec, pack_to_vec_at_version};
    pub use crate::versioned::{
        deserialize_in_place_versioned, deserialize_versioned, pack_size_versioned, pack_versioned,
        serialize_versioned, serialized_size_versioned, unpack_in_place_versioned,
        unpack_versioned,
    };
    pub use crate::{
        deserialize::{
            deserialize, deserialize_at_version, deserialize_in_place,
            deserialize_in_place_at_version,
        },
        packet::{
            pack, pack_at_version, pack_or_size, pack_or_size_at_version, pack_size,
            pack_size_at_version, pack_unchecked, pack_unchecked_at_version, read_pack_size,
            read_pack_size_at_version, unpack, unpack_at_version, unpack_in_place,
            unpack_in_place_at_version,
        },
        serialize::{
            serialize, serialize_at_version, serialize_or_size, serialize_or_size_at_version,
            serialize_unchecked, serialize_unchecked_at_version, serialized_size,
            serialized_size_at_version,
        },
    };

    #[cfg(feature = "alloc")]
    pub use crate::serialize::{serialize_to_vec, serialize_to_vec_at_version};
}

/// Module containing functions for serialization/deserialization that use
/// 1 byte for length and address fields, suitable for small data (up to 255B / 255 elements).
///
/// Common use case - short messages with bounded small footprint such as commands transmitted over network.
pub mod small {
    #[cfg(feature = "alloc")]
    pub use crate::packet::small::{pack_to_vec, pack_to_vec_at_version};
    pub use crate::versioned::small::{
        deserialize_in_place_versioned, deserialize_versioned, pack_size_versioned, pack_versioned,
        serialize_versioned, serialized_size_versioned, unpack_in_place_versioned,
        unpack_versioned,
    };
    pub use crate::{
        deserialize::small::{
            deserialize, deserialize_at_version, deserialize_in_place,
            deserialize_in_place_at_version,
        },
        packet::small::{
            pack, pack_at_version, pack_or_size, pack_or_size_at_version, pack_size,
            pack_size_at_version, pack_unchecked, pack_unchecked_at_version, read_pack_size,
            read_pack_size_at_version, unpack, unpack_at_version, unpack_in_place,
            unpack_in_place_at_version,
        },
        serialize::small::{
            serialize, serialize_at_version, serialize_or_size, serialize_or_size_at_version,
            serialize_unchecked, serialize_unchecked_at_version, serialized_size,
            serialized_size_at_version,
        },
    };

    #[cfg(feature = "alloc")]
    pub use crate::serialize::small::{serialize_to_vec, serialize_to_vec_at_version};
}

/// Module containing functions for serialization/deserialization that use
/// 2 bytes for length and address fields, suitable for medium data (up to 65KB / 65K elements).
///
/// Common use case - normal-sized packets transmitted over network.
pub mod medium {
    #[cfg(feature = "alloc")]
    pub use crate::packet::medium::{pack_to_vec, pack_to_vec_at_version};
    pub use crate::versioned::medium::{
        deserialize_in_place_versioned, deserialize_versioned, pack_size_versioned, pack_versioned,
        serialize_versioned, serialized_size_versioned, unpack_in_place_versioned,
        unpack_versioned,
    };
    pub use crate::{
        deserialize::medium::{
            deserialize, deserialize_at_version, deserialize_in_place,
            deserialize_in_place_at_version,
        },
        packet::medium::{
            pack, pack_at_version, pack_or_size, pack_or_size_at_version, pack_size,
            pack_size_at_version, pack_unchecked, pack_unchecked_at_version, read_pack_size,
            read_pack_size_at_version, unpack, unpack_at_version, unpack_in_place,
            unpack_in_place_at_version,
        },
        serialize::medium::{
            serialize, serialize_at_version, serialize_or_size, serialize_or_size_at_version,
            serialize_unchecked, serialize_unchecked_at_version, serialized_size,
            serialized_size_at_version,
        },
    };

    #[cfg(feature = "alloc")]
    pub use crate::serialize::medium::{serialize_to_vec, serialize_to_vec_at_version};
}

/// Module containing functions for serialization/deserialization that use
/// 4 bytes for length and address fields, suitable for large data (up to 4GB / 4B elements).
///
/// Common use case - data stored in files.
pub mod large {
    #[cfg(feature = "alloc")]
    pub use crate::packet::large::{pack_to_vec, pack_to_vec_at_version};
    pub use crate::versioned::large::{
        deserialize_in_place_versioned, deserialize_versioned, pack_size_versioned, pack_versioned,
        serialize_versioned, serialized_size_versioned, unpack_in_place_versioned,
        unpack_versioned,
    };
    pub use crate::{
        deserialize::large::{
            deserialize, deserialize_at_version, deserialize_in_place,
            deserialize_in_place_at_version,
        },
        packet::large::{
            pack, pack_at_version, pack_or_size, pack_or_size_at_version, pack_size,
            pack_size_at_version, pack_unchecked, pack_unchecked_at_version, read_pack_size,
            read_pack_size_at_version, unpack, unpack_at_version, unpack_in_place,
            unpack_in_place_at_version,
        },
        serialize::large::{
            serialize, serialize_at_version, serialize_or_size, serialize_or_size_at_version,
            serialize_unchecked, serialize_unchecked_at_version, serialized_size,
            serialized_size_at_version,
        },
    };

    #[cfg(feature = "alloc")]
    pub use crate::serialize::large::{serialize_to_vec, serialize_to_vec_at_version};
}

/// Module containing functions for serialization/deserialization that use
/// 8 bytes for length and address fields, suitable for huge data (up to 16EB / 18 quintillion elements).
///
/// Common use case - data stored in large files exceeding 4GB.
pub mod huge {
    #[cfg(feature = "alloc")]
    pub use crate::packet::huge::{pack_to_vec, pack_to_vec_at_version};
    pub use crate::versioned::huge::{
        deserialize_in_place_versioned, deserialize_versioned, pack_size_versioned, pack_versioned,
        serialize_versioned, serialized_size_versioned, unpack_in_place_versioned,
        unpack_versioned,
    };
    pub use crate::{
        deserialize::huge::{
            deserialize, deserialize_at_version, deserialize_in_place,
            deserialize_in_place_at_version,
        },
        packet::huge::{
            pack, pack_at_version, pack_or_size, pack_or_size_at_version, pack_size,
            pack_size_at_version, pack_unchecked, pack_unchecked_at_version, read_pack_size,
            read_pack_size_at_version, unpack, unpack_at_version, unpack_in_place,
            unpack_in_place_at_version,
        },
        serialize::huge::{
            serialize, serialize_at_version, serialize_or_size, serialize_or_size_at_version,
            serialize_unchecked, serialize_unchecked_at_version, serialized_size,
            serialized_size_at_version,
        },
    };

    #[cfg(feature = "alloc")]
    pub use crate::serialize::huge::{serialize_to_vec, serialize_to_vec_at_version};
}

/// Module containing functions for serialization/deserialization that use
/// 16 bytes for length and address fields, suitable for humongous data (up to 256 UB / 340 undecillion elements).
///
/// Common use case - on today data scale no single message would overflow 16EB of [`huge`] category,
/// but why not prepare for the future when 17EB of data may be transmitted in a single message, or stored in a single file?
pub mod humongous {
    #[cfg(feature = "alloc")]
    pub use crate::packet::humongous::{pack_to_vec, pack_to_vec_at_version};
    pub use crate::versioned::humongous::{
        deserialize_in_place_versioned, deserialize_versioned, pack_size_versioned, pack_versioned,
        serialize_versioned, serialized_size_versioned, unpack_in_place_versioned,
        unpack_versioned,
    };
    pub use crate::{
        deserialize::humongous::{
            deserialize, deserialize_at_version, deserialize_in_place,
            deserialize_in_place_at_version,
        },
        packet::humongous::{
            pack, pack_at_version, pack_or_size, pack_or_size_at_version, pack_size,
            pack_size_at_version, pack_unchecked, pack_unchecked_at_version, read_pack_size,
            read_pack_size_at_version, unpack, unpack_at_version, unpack_in_place,
            unpack_in_place_at_version,
        },
        serialize::humongous::{
            serialize, serialize_at_version, serialize_or_size, serialize_or_size_at_version,
            serialize_unchecked, serialize_unchecked_at_version, serialized_size,
            serialized_size_at_version,
        },
    };

    #[cfg(feature = "alloc")]
    pub use crate::serialize::humongous::{serialize_to_vec, serialize_to_vec_at_version};
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
        element::Element,
        formula::{Formula, SizeBound},
        serialize::{Serialize, Serializer, Sizes},
    };

    pub use core::{
        marker::PhantomData,
        option::Option::{self, None, Some},
        result::Result::{self, Err, Ok},
    };

    pub use {bool, f32, f64, i8, i16, i32, i64, i128, u8, u16, u32, u64, u128};

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

    /// Select a field's element and explicit version for generated codecs.
    #[inline(always)]
    pub fn with_element_versioned<F: ?Sized, E: Element<VERSION> + ?Sized, const VERSION: u32>(
        f: impl FnOnce(&F) -> &E,
    ) -> WithElement<E, VERSION> {
        let _ = f;
        WithElement {
            _marker: core::marker::PhantomData,
        }
    }

    pub struct WithElement<E: ?Sized, const VERSION: u32 = 0> {
        _marker: core::marker::PhantomData<E>,
    }

    impl<E, const VERSION: u32> WithElement<E, VERSION>
    where
        E: Element<VERSION> + ?Sized,
    {
        /// Helper function to take element formula from a composite formula.
        /// And then use it to serialize element either directly.
        #[inline(always)]
        pub fn serialize<T, S>(self, value: &T, serializer: &mut S) -> Result<(), S::Error>
        where
            T: Serialize<E::Formula, VERSION> + ?Sized,
            S: Serializer,
        {
            E::serialize(value, serializer)
        }

        /// Helper function to take element formula from a composite formula.
        /// And then use it to get size hint for element direct serialization.
        #[inline(always)]
        pub fn size_hint<T, const SIZE_BYTES: usize>(self, value: &T) -> Option<Sizes>
        where
            T: Serialize<E::Formula, VERSION> + ?Sized,
        {
            crate::advanced::size_hint_versioned::<E, T, SIZE_BYTES, VERSION>(value)
        }

        /// Size hint including stack padding before the next field.
        #[inline(always)]
        pub fn size_hint_padded<T, const SIZE_BYTES: usize>(self, value: &T) -> Option<Sizes>
        where
            T: Serialize<E::Formula, VERSION> + ?Sized,
        {
            let mut sizes = simple_some!(crate::advanced::size_hint_versioned::<
                E,
                T,
                SIZE_BYTES,
                VERSION,
            >(value));
            if let SizeBound::Bounded(max_stack) =
                const { crate::stack_size_versioned::<E, SIZE_BYTES, VERSION>() }
            {
                sizes.stack = max_stack;
            }
            Some(sizes)
        }

        #[inline(always)]
        pub fn deserialize<'de, T, D>(self, deserializer: &mut D) -> Result<T, DeserializeError>
        where
            T: Deserialize<'de, E::Formula, VERSION>,
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
            T: Deserialize<'de, E::Formula, VERSION> + ?Sized,
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
            0x10000..=0xFFFFFF => 3,
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

    pub trait DeserializeEnumVariant<'de, F: Formula<VERSION> + ?Sized, const VERSION: u32 = 0> {
        fn deserialize_enum_variant<D>(
            discriminant: usize,
            deserializer: D,
        ) -> Result<Self, DeserializeError>
        where
            D: Deserializer<'de>,
            Self: Sized;
    }
}
