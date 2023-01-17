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

mod array;
mod bytes;
mod option;
mod primitive;
mod schema;
mod seq;
mod str;
mod tuple;

use core::{mem::size_of, num::TryFromIntError};

pub use self::schema::{Access, Schema, Serialize};

#[cfg(feature = "derive")]
pub use alkahest_proc::Schema;

pub use self::{
    bytes::{Bytes, BytesHeader},
    seq::{Seq, SeqAccess, SeqHeader, SeqIter},
    str::Str,
};

/// Serializes data into bytes slice.
/// Returns number of bytes written.
///
/// # Panics
///
/// Panics if value doesn't fit into bytes.
#[inline(always)]
pub fn serialize<T, S>(serializable: S, output: &mut [u8]) -> Result<usize, usize>
where
    T: Schema,
    S: Serialize<T>,
{
    if output.len() < T::header() {
        return Err(T::header() + serializable.body_size());
    }

    let (head, tail) = output.split_at_mut(T::header());

    match serializable.serialize_body(tail) {
        Ok((header, size)) => {
            let total = size + T::header();
            if S::serialize_header(header, head, T::header()) {
                Ok(total)
            } else {
                Err(total)
            }
        }
        Err(size) => Err(T::header() + size),
    }
}

/// Deserializes data from byte slice.
#[inline(always)]
pub fn access<'a, T>(input: &'a [u8]) -> Access<'a, T>
where
    T: Schema,
{
    T::access(input)
}

type FixedUsizeType = u32;

/// Type used to represent sizes and offsets in serialized data.
/// This places limitation on sequence sizes which practically is never hit.
/// `usize` itself is not portable and cannot be written into alkahest package.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct FixedUsize(FixedUsizeType);

impl FixedUsize {
    #[inline(always)]
    fn truncated(value: usize) -> Self {
        FixedUsize(value as FixedUsizeType)
    }

    #[inline(always)]
    fn to_bytes(self) -> [u8; size_of::<Self>()] {
        self.0.to_le_bytes()
    }

    #[inline(always)]
    fn from_bytes(bytes: [u8; size_of::<Self>()]) -> Self {
        FixedUsize(FixedUsizeType::from_le_bytes(bytes))
    }
}

impl TryFrom<usize> for FixedUsize {
    type Error = TryFromIntError;

    #[inline(always)]
    fn try_from(value: usize) -> Result<Self, TryFromIntError> {
        FixedUsizeType::try_from(value).map(FixedUsize)
    }
}

impl TryFrom<FixedUsizeType> for FixedUsize {
    type Error = TryFromIntError;

    #[inline(always)]
    fn try_from(value: FixedUsizeType) -> Result<Self, TryFromIntError> {
        usize::try_from(value)?;
        Ok(FixedUsize(value))
    }
}

impl From<FixedUsize> for usize {
    #[inline(always)]
    fn from(value: FixedUsize) -> Self {
        value.0 as usize
    }
}

impl From<FixedUsize> for FixedUsizeType {
    #[inline(always)]
    fn from(value: FixedUsize) -> Self {
        value.0
    }
}

#[doc(hidden)]
pub mod private {
    use core::mem::size_of;

    pub use {bool, u32, u8, usize, Result};

    pub const VARIANT_SIZE: usize = size_of::<u32>();

    #[inline(always)]
    pub fn write_variant_index(
        variant: u32,
        output: &mut [u8],
        offset: usize,
    ) -> (&mut [u8], usize) {
        output[..VARIANT_SIZE].copy_from_slice(&variant.to_le_bytes());
        (&mut output[VARIANT_SIZE..], offset - VARIANT_SIZE)
    }

    #[inline(always)]
    pub fn read_variant_index(input: &[u8]) -> (&[u8], u32) {
        let (head, tail) = input.split_at(VARIANT_SIZE);
        let variant = u32::from_le_bytes(head.try_into().unwrap());
        (tail, variant)
    }
}
