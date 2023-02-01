//!
//! *Alkahest* is serialization library with blazing fast serialization and zero-overhead deserialization.
//! It is designed to be used in network protocols and other high-performance applications.
//!
//! *Alkahest* uses Rust procedure-macros for code generation instead of relying on external tools.
//! Works out-of-the-box in Rust ecosystem.
//! As of now it does not support other languages.
//!
//! *Alkahest* uses types that implement `Schema` trait to serialize and access data.
//! Unlike other serialization libraries, type that implements `Schema` is not a data container.
//! Serialization works by packing data using `Pack<SchemaType>` trait, implemented for fundamental types.
//! Primitives can be packed from themselves and more restrictive types basic types.
//! Sequences are packed from anything that can be iterated over with items that can be packed into sequence element.
//! Arrays are packed from arrays of types that can be packed into array element.
//! For user-defined `SchemaType`, `Pack<SchemaType>` is implemented for types generated using `Schema` derive macro.
//! For structs `Pack<SchemaType>` is implemented for struct with same fields but where all field types are disticnt generic parameter.
//! For enums `Pack<SchemaType>` is implemented for struct generated for each enum variant otherwise similar to struct.
//!
//! Deserialization works by reading data from bytes. Streaming deserialization is not yet supported.
//! On deserialization only highest-level data is Access and the rest is read only on access to returned value.
//! Types are Access by casting byte array where possible making it zero-copy in this case.
//!

#![no_std]
#![deny(unsafe_code)]

extern crate self as alkahest;

#[cfg(feature = "alloc")]
extern crate alloc;

/// This macro allows to define new schema types
/// as an alias to existing schema types.
///
/// `Serialize` and `Deserialize` implementation
/// will use aliased schema.
#[macro_export]
macro_rules! schema_alias {
    ($(+[$($p:tt)*])? $a:ty as $b:ty $(where $($wc:tt)*)?) => {
        impl $(< $($p)* >)? $crate::Schema for $a
        where
            $b: $crate::Schema
            $($($wc)*)?
        {}

        impl< $($($p)*,)? __Serializable > $crate::Serialize<$a> for __Serializable
        where
            $b: $crate::Schema,
            Self: $crate::Serialize<$b>,
        {
            fn serialize(self, offset: $crate::private::usize, output: &mut [$crate::private::u8]) -> $crate::private::Result<($crate::private::usize, $crate::private::usize), $crate::private::usize> {
                <Self as $crate::Serialize<$b>>::serialize(self, offset, output)
            }

            fn size(self) -> $crate::private::usize {
                <Self as $crate::Serialize<$b>>::size(self)
            }
        }

        impl<'__de,  $($($p)*,)? __Deserializable > $crate::Deserialize<'__de, $a> for __Deserializable
        where
            $b: $crate::Schema,
            Self: $crate::Deserialize<'__de, $b>,
        {
            fn deserialize(len: $crate::private::usize, input: &'__de [$crate::private::u8]) -> $crate::private::Result<Self, $crate::DeserializeError> {
                <Self as $crate::Deserialize<'__de, $b>>::deserialize(len, input)
            }

            fn deserialize_in_place(&mut self, len: $crate::private::usize, input: &'__de [$crate::private::u8]) -> $crate::private::Result<(), $crate::DeserializeError> {
                <Self as $crate::Deserialize<'__de, $b>>::deserialize_in_place(self, len, input)
            }
        }
    };

    (@sized $(+[$($p:tt)*])? $a:ty as $b:ty $(where $($wc:tt)*)?) => {
        schema_alias!($(+[$($p)*])? $a as $b $(where $($wc)*)?);

        impl $(< $($p)* >)? $crate::SizedSchema for $a
        where
            $b: $crate::SizedSchema
            $(where $($wc)*)?
        {
            const SIZE: $crate::private::usize = <$b as $crate::SizedSchema>::SIZE;
        }
    };
}

mod array;
// mod bytes;
mod deserialize;
// mod option;
mod primitive;
mod reference;
mod schema;
// mod seq;
mod serialize;
// mod str;
mod size;
mod slice;
mod tuple;

#[cfg(feature = "alloc")]
mod vec;

pub use self::{
    deserialize::{deserialize, Deserialize, DeserializeError, Deserializer},
    reference::Ref,
    schema::{Schema, SizedSchema},
    serialize::{serialize, serialized_size, Serialize, Serializer},
    slice::SliceIter,
};

#[cfg(feature = "derive")]
pub use alkahest_proc::{Deserialize, Schema, Serialize, SizedSchema};

#[doc(hidden)]
pub mod private {
    pub use {bool, u32, u8, usize, Result};

    use core::marker::PhantomData;

    use crate::SizedSchema;
    pub use crate::{Deserialize, DeserializeError, Deserializer, Schema, Serialize, Serializer};

    pub struct WithSchema<S: ?Sized> {
        marker: PhantomData<fn(&S) -> &S>,
    }

    impl<S> WithSchema<S>
    where
        S: Schema + ?Sized,
    {
        pub fn serialize_value<T>(self, ser: &mut Serializer, value: T) -> Result<(), usize>
        where
            T: Serialize<S>,
        {
            ser.serialize_value::<S, T>(value)
        }

        pub fn size_value<T>(self, value: T) -> usize
        where
            T: Serialize<S>,
        {
            <T as Serialize<S>>::size(value)
        }

        pub fn deserialize_sized<'de, T>(
            self,
            des: &mut Deserializer<'de>,
        ) -> Result<T, DeserializeError>
        where
            S: SizedSchema,
            T: Deserialize<'de, S>,
        {
            des.deserialize_sized::<S, T>()
        }

        pub fn deserialize_rest<'de, T>(
            self,
            des: &mut Deserializer<'de>,
        ) -> Result<T, DeserializeError>
        where
            S: Schema,
            T: Deserialize<'de, S>,
        {
            des.deserialize_rest::<S, T>()
        }
    }

    pub fn with_schema<S: ?Sized, F: ?Sized>(_: impl FnOnce(&S) -> &F) -> WithSchema<F> {
        WithSchema {
            marker: PhantomData,
        }
    }
}
