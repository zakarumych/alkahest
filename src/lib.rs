//!
//! *Alkahest* is serialization library aimed for packet writing and reading in hot path.
//! For this purpose *Alkahest* avoids allocations and reads data only on demand.
//!
//! Key differences of *Alkahest* from other popular serialization crates is zero-overhead serialization and zero-copy lazy deserialization.\
//! For example to serialize value sequence it is not necessary to construct expensive type with allocations such as vectors.\
//! Instead sequences are serialized directly from iterators. On deserialization an iterator is returned to the user, which does not parse any element before it is requested.
//! Which means that data that is not accessed - not parsed either.
//!
//! *Alkahest* works similarly to *FlatBuffers*,\
//! but does not require using another language for data scheme definition and running external tool,\
//! and supports generic schemas.
//!

#![no_std]
#![deny(unsafe_code)]

#[cfg(feature = "alloc")]
extern crate alloc;

mod schema;

#[cfg(feature = "nightly")]
mod array;
mod bytes;
mod option;
mod primitive;
mod seq;
mod str;
mod tuple;

use core::mem::size_of;

pub use self::{
    bytes::Bytes,
    schema::{Pack, Packed, Schema, SchemaOwned, SchemaUnpack, Unpacked},
    seq::{Seq, SeqIter, SeqUnpacked},
    str::Str,
};

#[cfg(feature = "derive")]
pub use alkahest_proc::Schema;

// Exports for proc-macro.
#[doc(hidden)]
pub use bytemuck::{Pod, Zeroable};

/// Writes data into bytes slice.
/// Returns number of bytes written.
///
/// # Panics
///
/// Panics if value doesn't fit into bytes.
pub fn write<'a, T, P>(bytes: &'a mut [u8], packable: P) -> usize
where
    T: Schema,
    P: Pack<T>,
{
    let align_mask = T::align() - 1;
    debug_assert_eq!(
        bytes.as_ptr() as usize & align_mask,
        0,
        "Output is not aligned to {}",
        align_mask + 1
    );

    let packed_size = size_of::<T::Packed>();
    let aligned = (packed_size + align_mask) & !align_mask;
    let (packed, used) = packable.pack(aligned, &mut bytes[aligned..]);
    bytes[..packed_size].copy_from_slice(bytemuck::bytes_of(&packed));
    aligned + used
}

/// Reads and unpacks package from raw bytes.
///
/// # Panics
///
/// This function or returned value's methods may panic
/// if `bytes` slice does not contain data written with same schema.
pub fn read<'a, T>(bytes: &'a [u8]) -> Unpacked<'a, T>
where
    T: Schema,
{
    T::unpack(
        *bytemuck::from_bytes(&bytes[..size_of::<T::Packed>()]),
        bytes,
    )
}

/// Type used to represent sizes and offsets in alkahest packages.
/// This places limitation on sequence sizes which practically is never hit.
/// `usize` itself is not portable and cannot be written into alkahest package.
#[doc(hidden)]
pub type FixedUsize = u32;
