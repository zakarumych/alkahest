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

#[cfg(feature = "panicking")]
#[macro_export]
macro_rules! cold_panic {
    ($($arg:tt)*) => {{
        #[cold]
        #[inline(never)]
        fn do_cold_panic() -> ! {
            panic!($($arg)*);
        }
        do_cold_panic()
    }};
}

// mod array;
// mod bytes;
mod deserialize;
// mod option;
mod primitive;
// mod reference;
mod schema;
// mod seq;
mod serialize;
// mod str;
// mod tuple;
mod size;

// // pub use self::{schema::{Access, Schema}, serialize::Serialize};

// #[cfg(feature = "derive")]
// pub use alkahest_proc::Schema;

// pub use self::{
//     bytes::{Bytes, BytesHeader},
//     seq::{Seq, SeqAccess, SeqHeader, SeqIter},
//     str::Str,
// };

// /// Calculates size of serialized data.
// ///
// /// # Examples
// ///
// /// ```
// /// use alkahest::{Schema, Serialize, bytes_size, Seq};
// ///
// /// #[derive(Schema)]
// /// struct MySchema {
// ///   a: u8,
// ///   b: u16,
// ///   c: Seq<u32>,
// /// }
// ///
// /// let expected_size = 1 + 2 + 4 * 2 + 4 * 3; // a - 1 byte, b - 2 bytes, c - 2 u32s for header + 3 u32s
// ///
// /// let size = bytes_size::<MySchema, _>(MySchemaSerialize {
// ///   a: 1,
// ///   b: 2,
// ///   c: 3..6,
// /// });
// ///
// /// assert_eq!(size, expected_size);
// /// ```
// #[inline(always)]
// pub fn bytes_size<T, S>(serializable: S) -> usize
// where
//     T: schema::Schema,
//     S: Serialize<T>,
// {
//     T::header() + serializable.body_size()
// }

// /// Access data from byte slice.
// ///
// /// Returns value that can be used to traverse data according to specified schema down to primitive types.
// ///
// // TODO: Add fallible version of this function - `try_access`.
// #[inline(always)]
// pub fn access<'a, T>(input: &'a [u8]) -> Access<'a, T>
// where
//     T: Schema,
// {
//     T::access(input)
// }

// #[doc(hidden)]
// pub mod private {
//     use core::mem::size_of;

//     pub use {bool, u32, u8, usize, Result};

//     pub const VARIANT_SIZE: usize = size_of::<u32>();

//     #[inline(always)]
//     pub fn write_variant_index(
//         variant: u32,
//         output: &mut [u8],
//         offset: usize,
//     ) -> (&mut [u8], usize) {
//         output[..VARIANT_SIZE].copy_from_slice(&variant.to_le_bytes());
//         (&mut output[VARIANT_SIZE..], offset - VARIANT_SIZE)
//     }

//     #[inline(always)]
//     pub fn read_variant_index(input: &[u8]) -> (&[u8], u32) {
//         let (head, tail) = input.split_at(VARIANT_SIZE);
//         let variant = u32::from_le_bytes(head.try_into().unwrap());
//         (tail, variant)
//     }
// }
