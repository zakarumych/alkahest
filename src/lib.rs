#![no_std]
#![deny(unsafe_code)]

extern crate self as alkahest;

#[cfg(feature = "alloc")]
extern crate alloc;

mod array;
mod bytes;
mod deserialize;
mod formula;
mod lazy;
mod option;
mod primitive;
mod reference;
mod serialize;
mod size;
mod skip;
mod slice;
mod tuple;

#[cfg(feature = "alloc")]
mod vec;

pub use self::{
    bytes::Bytes,
    deserialize::{deserialize, value_size, Deserialize, Deserializer, Error},
    formula::Formula,
    lazy::Lazy,
    reference::Ref,
    serialize::{serialize, serialize_ff, serialized_size, Serialize, Serializer},
    skip::Skip,
    slice::SliceIter,
};

#[cfg(feature = "derive")]
pub use alkahest_proc::{Deserialize, Formula, Serialize, UnsizedFormula};

// /// Private module for macros to use.
// #[cfg(feature = "derive")]
// #[doc(hidden)]
// pub mod private {
//     pub use {bool, u32, u8, usize, Result};

//     use core::marker::PhantomData;

//     pub use crate::{
//         deserialize::{Deserialize, Error, Deserializer},
//         formula::{Formula, UnsizedFormula},
//         serialize::{Serialize, Serializer},
//     };

//     pub struct WithFormula<F: UnsizedFormula + ?Sized> {
//         marker: PhantomData<fn(&F) -> &F>,
//     }

//     impl<F> WithFormula<F>
//     where
//         F: UnsizedFormula + ?Sized,
//     {
//         pub fn serialize_sized<T>(self, ser: &mut Serializer, value: T) -> Result<(), usize>
//         where
//             F: Formula + ?Sized,
//             T: Serialize<F>,
//         {
//             ser.serialize_sized::<F, T>(value)
//         }

//         pub fn serialize_unsized<T>(self, ser: &mut Serializer, value: T) -> Result<(), usize>
//         where
//             T: Serialize<F>,
//         {
//             ser.serialize_sized::<F, T>(value)
//         }

//         pub fn size_value<T>(self, value: T) -> usize
//         where
//             T: Serialize<F>,
//         {
//             <T as Serialize<F>>::size(value)
//         }

//         pub fn deserialize_sized<'de, T>(
//             self,
//             des: &mut Deserializer<'de>,
//         ) -> Result<T, Error>
//         where
//             F: Formula,
//             T: Deserialize<'de, F>,
//         {
//             des.deserialize_sized::<F, T>()
//         }

//         pub fn deserialize_rest<'de, T>(
//             self,
//             des: &mut Deserializer<'de>,
//         ) -> Result<T, Error>
//         where
//             F: Formula,
//             T: Deserialize<'de, F>,
//         {
//             des.deserialize_rest::<F, T>()
//         }
//     }

//     pub fn with_formula<F: Formula + ?Sized, L: Formula + ?Sized>(
//         _: impl FnOnce(&F) -> &L,
//     ) -> WithFormula<L> {
//         WithFormula {
//             marker: PhantomData,
//         }
//     }
// }
