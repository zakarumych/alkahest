use core::mem::size_of;

use crate::{
    buffer::Buffer,
    deserialize::{Deserialize, DeserializeError, Deserializer},
    formula::{BareFormulaType, FormulaType},
    serialize::{write_bytes, Serialize, Sizes},
};

cfg_if::cfg_if! {
    if #[cfg(feature = "fixed64")] {
        /// Type used to represent sizes and offsets in serialized data.
        pub type FixedUsizeType = u64;
    } else if #[cfg(feature = "fixed32")] {
        /// Type used to represent sizes and offsets in serialized data.
        pub type FixedUsizeType = u32;
    } else if #[cfg(feature = "fixed16")] {
        /// Type used to represent sizes and offsets in serialized data.
        pub type FixedUsizeType = u16;
    } else if #[cfg(feature = "fixed8")] {
        /// Type used to represent sizes and offsets in serialized data.
        pub type FixedUsizeType = u8;
    } else {
        compile_error!("No fixed size integer feature enabled");
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "fixed64")] {
        /// Type used to represent sizes and offsets in serialized data.
        pub type FixedIsizeType = i64;
    } else if #[cfg(feature = "fixed32")] {
        /// Type used to represent sizes and offsets in serialized data.
        pub type FixedIsizeType = i32;
    } else if #[cfg(feature = "fixed16")] {
        /// Type used to represent sizes and offsets in serialized data.
        pub type FixedIsizeType = i16;
    } else if #[cfg(feature = "fixed8")] {
        /// Type used to represent sizes and offsets in serialized data.
        pub type FixedIsizeType = i8;
    } else {
        compile_error!("No fixed size integer feature enabled");
    }
}

pub const SIZE_STACK: usize = size_of::<FixedUsizeType>();

pub fn usize_truncate_unchecked(value: usize) -> FixedUsizeType {
    debug_assert!(FixedUsizeType::try_from(value).is_ok());
    value as FixedUsizeType
}

pub fn isize_truncate_unchecked(value: isize) -> FixedIsizeType {
    debug_assert!(FixedIsizeType::try_from(value).is_ok());
    value as FixedIsizeType
}

impl FormulaType for usize {
    const MAX_STACK_SIZE: Option<usize> = Some(size_of::<FixedUsizeType>());
    const EXACT_SIZE: bool = true;
    const HEAPLESS: bool = true;
}

impl BareFormulaType for usize {}

impl Serialize<usize> for usize {
    #[inline(always)]
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        serialize_usize(self, sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        Some(Sizes::with_stack(size_of::<FixedUsizeType>()))
    }
}

impl Serialize<usize> for &usize {
    #[inline(always)]
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        serialize_usize(*self, sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        Some(Sizes::with_stack(size_of::<FixedUsizeType>()))
    }
}

impl Deserialize<'_, usize> for usize {
    #[inline(always)]
    fn deserialize(de: Deserializer) -> Result<Self, DeserializeError> {
        deserialize_usize(de)
    }

    #[inline(always)]
    fn deserialize_in_place(&mut self, de: Deserializer) -> Result<(), DeserializeError> {
        *self = deserialize_usize(de)?;
        Ok(())
    }
}

impl FormulaType for isize {
    const MAX_STACK_SIZE: Option<usize> = Some(size_of::<FixedIsizeType>());
    const EXACT_SIZE: bool = true;
    const HEAPLESS: bool = true;
}

impl BareFormulaType for isize {}

impl Serialize<isize> for isize {
    #[inline(always)]
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        serialize_isize(self, sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        Some(Sizes::with_stack(size_of::<FixedIsizeType>()))
    }
}

impl Serialize<isize> for &isize {
    #[inline(always)]
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        serialize_isize(*self, sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        Some(Sizes::with_stack(size_of::<FixedIsizeType>()))
    }
}

impl Deserialize<'_, isize> for isize {
    #[inline(always)]
    fn deserialize(de: Deserializer) -> Result<Self, DeserializeError> {
        deserialize_isize(de)
    }

    #[inline(always)]
    fn deserialize_in_place(&mut self, de: Deserializer) -> Result<(), DeserializeError> {
        *self = deserialize_isize(de)?;
        Ok(())
    }
}

#[inline(always)]
pub fn serialize_usize<B>(value: usize, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
where
    B: Buffer,
{
    write_bytes(
        &usize_truncate_unchecked(value).to_le_bytes(),
        sizes,
        buffer,
    )
}

#[inline(always)]
pub fn serialize_isize<B>(value: isize, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
where
    B: Buffer,
{
    write_bytes(
        &isize_truncate_unchecked(value).to_le_bytes(),
        sizes,
        buffer,
    )
}

#[inline(always)]
pub fn deserialize_usize(mut de: Deserializer) -> Result<usize, DeserializeError> {
    let input = de.read_byte_array::<{ size_of::<FixedUsizeType>() }>()?;
    // de.finish()?;
    let value = <FixedUsizeType>::from_le_bytes(input);

    #[cfg(debug_assertions)]
    if usize::try_from(value).is_err() {
        return Err(DeserializeError::InvalidUsize(value));
    }

    Ok(value as usize)
}

#[inline(always)]
pub fn deserialize_isize(mut de: Deserializer) -> Result<isize, DeserializeError> {
    let input = de.read_byte_array::<{ size_of::<FixedIsizeType>() }>()?;
    // de.finish()?;
    let value = <FixedIsizeType>::from_le_bytes(input);

    #[cfg(debug_assertions)]
    if usize::try_from(value).is_err() {
        return Err(DeserializeError::InvalidIsize(value));
    }

    Ok(value as isize)
}
