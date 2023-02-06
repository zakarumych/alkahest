#![no_std]
#![deny(unsafe_code)]

extern crate self as alkahest;

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod array;
pub mod bytes;
pub mod deserialize;
pub mod formula;
pub mod lazy;
pub mod option;
pub mod primitive;
pub mod reference;
pub mod serialize;
pub mod size;
pub mod skip;
pub mod slice;
pub mod tuple;

#[cfg(feature = "alloc")]
pub mod vec;

pub mod prelude {
    pub use crate::{
        deserialize::{deserialize, deserialize_in_place, Deserialize, Error},
        formula::Formula,
        serialize::{serialize, serialize_or_size, serialized_size, Serialize, SerializeOwned},
    };

    #[cfg(feature = "derive")]
    pub use alkahest_proc::{Deserialize, Formula, Serialize};
}

#[cfg(feature = "derive")]
pub use alkahest_proc::{Deserialize, Formula, Serialize};

/// Private module for macros to use.
#[cfg(feature = "derive")]
#[doc(hidden)]
pub mod private {
    pub use {bool, u32, u8, usize, Into, Option, Result};

    use core::marker::PhantomData;

    pub use crate::{
        deserialize::{Deserialize, Deserializer, Error, NonRefDeserialize},
        formula::{combine_sizes, Formula, NonRefFormula},
        serialize::{NonRefSerialize, NonRefSerializeOwned, Serialize, SerializeOwned, Serializer},
    };

    pub struct WithFormula<F: Formula + ?Sized> {
        marker: PhantomData<fn(&F) -> &F>,
    }

    impl<F> WithFormula<F>
    where
        F: Formula + ?Sized,
    {
        #[inline(always)]
        pub fn write_value<T, S>(self, ser: &mut S, value: T) -> Result<(), S::Error>
        where
            S: Serializer,
            T: SerializeOwned<F>,
        {
            ser.write_value::<F, T>(value)
        }

        #[inline(always)]
        pub fn read_value<'de, T>(self, des: &mut Deserializer<'de>) -> Result<T, Error>
        where
            F: Formula,
            T: Deserialize<'de, F>,
        {
            des.read_value::<F, T>()
        }

        #[inline(always)]
        pub fn read_in_place<'de, T>(
            self,
            place: &mut T,
            des: &mut Deserializer<'de>,
        ) -> Result<(), Error>
        where
            F: Formula,
            T: Deserialize<'de, F>,
        {
            des.read_in_place::<F, T>(place)
        }
    }

    #[inline(always)]
    pub fn with_formula<F: Formula + ?Sized, L: Formula + ?Sized>(
        _: impl FnOnce(&F) -> &L,
    ) -> WithFormula<L> {
        WithFormula {
            marker: PhantomData,
        }
    }
}
