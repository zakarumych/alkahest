//!
//! *Alkahest* is serialization library with blazing fast serialization and zero-overhead deserialization.
//! It is designed to be used in network protocols and other high-performance applications.
//!
//! *Alkahest* uses Rust procedure-macros for code generation instead of relying on external tools.
//! Works out-of-the-box in Rust ecosystem.
//! As of now it does not support other languages.
//!
//! *Alkahest* uses types that implement `Formula` trait to serialize and access data.
//! Unlike other serialization libraries, type that implements `Formula` is not a data container.
//! Serialization works by packing data using `Pack<FormulaType>` trait, implemented for fundamental types.
//! Primitives can be packed from themselves and more restrictive types basic types.
//! Sequences are packed from anything that can be iterated over with items that can be packed into sequence element.
//! Arrays are packed from arrays of types that can be packed into array element.
//! For user-defined `FormulaType`, `Pack<FormulaType>` is implemented for types generated using `Formula` derive macro.
//! For structs `Pack<FormulaType>` is implemented for struct with same fields but where all field types are disticnt generic parameter.
//! For enums `Pack<FormulaType>` is implemented for struct generated for each enum variant otherwise similar to struct.
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

mod array;
// mod bytes;
mod deserialize;
// mod option;
mod formula;
mod primitive;
mod reference;
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
    formula::{Formula, UnsizedFormula},
    reference::Ref,
    serialize::{serialize, serialized_size, Serialize, Serializer},
    slice::SliceIter,
};

#[cfg(feature = "derive")]
pub use alkahest_proc::{Deserialize, Formula, Serialize, UnsizedFormula};

#[doc(hidden)]
pub mod private {
    pub use {bool, u32, u8, usize, Result};

    use core::marker::PhantomData;

    use crate::Formula;
    pub use crate::{
        Deserialize, DeserializeError, Deserializer, Serialize, Serializer, UnsizedFormula,
    };

    pub struct WithFormula<S: UnsizedFormula + ?Sized> {
        marker: PhantomData<fn(&S) -> &S>,
    }

    impl<S> WithFormula<S>
    where
        S: UnsizedFormula + ?Sized,
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
            S: Formula,
            T: Deserialize<'de, S>,
        {
            des.deserialize_sized::<S, T>()
        }

        pub fn deserialize_rest<'de, T>(
            self,
            des: &mut Deserializer<'de>,
        ) -> Result<T, DeserializeError>
        where
            S: UnsizedFormula,
            T: Deserialize<'de, S>,
        {
            des.deserialize_rest::<S, T>()
        }
    }

    pub fn with_formula<S: UnsizedFormula + ?Sized, F: UnsizedFormula + ?Sized>(
        _: impl FnOnce(&S) -> &F,
    ) -> WithFormula<F> {
        WithFormula {
            marker: PhantomData,
        }
    }
}
