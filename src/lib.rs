//!
//! Alkahest generates code to write and read packets.
//! Fast, correct, with low overhead and with simple API.
//!
#![no_std]
#![forbid(unsafe_code)]

#[cfg(feature = "alloc")]
extern crate alloc;

mod primitive;
// mod pod;
mod schema;
mod seq;

use core::mem::size_of;

pub use self::{
    schema::{Pack, Packed, Schema, SchemaUnpack, Unpacked},
    seq::Seq,
};

#[cfg(feature = "derive")]
pub use alkahest_proc::Schema;

// Exports for proc-macro.
#[doc(hidden)]
pub use bytemuck::{Pod, Zeroable};

/// Writes the package into provided bytes slice.
/// Returns number of bytes written.
pub fn write<'a, T, P>(bytes: &'a mut [u8], packable: P) -> usize
where
    T: Schema,
    P: Pack<T>,
{
    let packed_size = size_of::<T::Packed>();
    let (packed, used) = packable.pack(packed_size, &mut bytes[packed_size..]);
    bytes[..packed_size].copy_from_slice(bytemuck::bytes_of(&packed));
    packed_size + used
}

/// Reads and unpacks package from raw bytes.
pub fn read<'a, T>(bytes: &'a [u8]) -> Unpacked<'a, T>
where
    T: Schema,
{
    T::unpack(
        *bytemuck::from_bytes(&bytes[..size_of::<T::Packed>()]),
        bytes,
    )
}
