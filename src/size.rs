use core::{mem::size_of, num::TryFromIntError};

use crate::{
    buffer::Buffer,
    deserialize::{Deserialize, DeserializeError, Deserializer},
    formula::{BareFormula, Formula},
    serialize::{Serialize, Sizes},
};

/// Type used to represent sizes and offsets in serialized data.
#[cfg(feature = "fixed8")]
pub type FixedUsizeType = u8;

/// Type used to represent sizes and offsets in serialized data.
#[cfg(feature = "fixed16")]
pub type FixedUsizeType = u16;

/// Type used to represent sizes and offsets in serialized data.
#[cfg(feature = "fixed32")]
pub type FixedUsizeType = u32;

/// Type used to represent sizes and offsets in serialized data.
#[cfg(feature = "fixed64")]
pub type FixedUsizeType = u64;

/// Type used to represent sizes and offsets in serialized data.
#[cfg(feature = "fixed8")]
pub type FixedIsizeType = i8;

/// Type used to represent sizes and offsets in serialized data.
#[cfg(feature = "fixed16")]
pub type FixedIsizeType = i16;

/// Type used to represent sizes and offsets in serialized data.
#[cfg(feature = "fixed32")]
pub type FixedIsizeType = i32;

/// Type used to represent sizes and offsets in serialized data.
#[cfg(feature = "fixed64")]
pub type FixedIsizeType = i64;

/// Type used to represent sizes and offsets in serialized data.
/// This places limitation on sequence sizes which practically is never hit.
/// `usize` itself is not portable and cannot be written into alkahest package.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FixedUsize(FixedUsizeType);

impl FixedUsize {
    /// Truncates `usize` to `FixedUsizeType` without checking.
    #[inline(always)]
    pub fn truncate_unchecked(value: usize) -> Self {
        debug_assert!(FixedUsize::try_from(value).is_ok());
        FixedUsize(value as FixedUsizeType)
    }

    /// Converts to byte array in little endian.
    #[inline(always)]
    pub fn to_le_bytes(self) -> [u8; size_of::<Self>()] {
        self.0.to_le_bytes()
    }

    /// Converts from byte array in little endian.
    #[inline(always)]
    pub fn from_le_bytes(bytes: [u8; size_of::<Self>()]) -> Result<Self, TryFromIntError> {
        FixedUsizeType::from_le_bytes(bytes).try_into()
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

impl Formula for FixedUsize {
    const MAX_STACK_SIZE: Option<usize> = Some(size_of::<FixedUsizeType>());
    const EXACT_SIZE: bool = true;
    const HEAPLESS: bool = true;
}

impl BareFormula for FixedUsize {}

impl Serialize<FixedUsize> for FixedUsize {
    #[inline(always)]
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        <FixedUsizeType as Serialize<FixedUsizeType>>::serialize(self.0, sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        Some(Sizes::with_stack(size_of::<FixedUsizeType>()))
    }
}

impl Serialize<FixedUsize> for &FixedUsize {
    #[inline(always)]
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        <FixedUsizeType as Serialize<FixedUsizeType>>::serialize(self.0, sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        Some(Sizes::with_stack(size_of::<FixedUsizeType>()))
    }
}

impl Serialize<FixedUsize> for usize {
    #[inline(always)]
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        Serialize::<FixedUsizeType>::serialize(
            FixedUsize::truncate_unchecked(self).0,
            sizes,
            buffer,
        )
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        Some(Sizes::with_stack(size_of::<FixedUsizeType>()))
    }
}

impl Serialize<FixedUsize> for &usize {
    #[inline(always)]
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        Serialize::<FixedUsizeType>::serialize(
            FixedUsize::truncate_unchecked(*self).0,
            sizes,
            buffer,
        )
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        Some(Sizes::with_stack(size_of::<FixedUsizeType>()))
    }
}

impl Deserialize<'_, FixedUsize> for FixedUsize {
    #[inline(always)]
    fn deserialize(de: Deserializer) -> Result<Self, DeserializeError> {
        let value = <FixedUsizeType as Deserialize<FixedUsizeType>>::deserialize(de)?;

        #[cfg(debug_assertions)]
        if usize::try_from(value).is_err() {
            return Err(DeserializeError::InvalidUsize(value));
        }

        Ok(FixedUsize(value))
    }

    #[inline(always)]
    fn deserialize_in_place(&mut self, de: Deserializer) -> Result<(), DeserializeError> {
        <FixedUsizeType as Deserialize<FixedUsizeType>>::deserialize_in_place(&mut self.0, de)
    }
}

impl Deserialize<'_, FixedUsize> for usize {
    #[inline(always)]
    fn deserialize(de: Deserializer) -> Result<Self, DeserializeError> {
        let value = <FixedUsizeType as Deserialize<FixedUsizeType>>::deserialize(de)?;

        #[cfg(debug_assertions)]
        if usize::try_from(value).is_err() {
            return Err(DeserializeError::InvalidUsize(value));
        }

        Ok(value as usize)
    }

    #[inline(always)]
    fn deserialize_in_place(&mut self, de: Deserializer) -> Result<(), DeserializeError> {
        *self = <Self as Deserialize<FixedUsize>>::deserialize(de)?;
        Ok(())
    }
}

/// Type used to represent sizes and offsets in serialized data.
/// This places limitation on sequence sizes which practically is never hit.
/// `usize` itself is not portable and cannot be written into alkahest package.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FixedIsize(FixedIsizeType);

impl FixedIsize {
    /// Truncates `isize` to `FixedIsizeType` without checking.
    #[inline(always)]
    pub fn truncate_unchecked(value: isize) -> Self {
        debug_assert!(FixedIsize::try_from(value).is_ok());
        FixedIsize(value as FixedIsizeType)
    }

    /// Converts to byte array in little endian.
    #[inline(always)]
    pub fn to_le_bytes(self) -> [u8; size_of::<Self>()] {
        self.0.to_le_bytes()
    }

    /// Converts from byte array in little endian.
    #[inline(always)]
    pub fn from_le_bytes(bytes: [u8; size_of::<Self>()]) -> Result<Self, TryFromIntError> {
        FixedIsizeType::from_le_bytes(bytes).try_into()
    }
}

impl TryFrom<isize> for FixedIsize {
    type Error = TryFromIntError;

    #[inline(always)]
    fn try_from(value: isize) -> Result<Self, TryFromIntError> {
        FixedIsizeType::try_from(value).map(FixedIsize)
    }
}

impl TryFrom<FixedIsizeType> for FixedIsize {
    type Error = TryFromIntError;

    #[inline(always)]
    fn try_from(value: FixedIsizeType) -> Result<Self, TryFromIntError> {
        isize::try_from(value)?;
        Ok(FixedIsize(value))
    }
}

impl From<FixedIsize> for isize {
    #[inline(always)]
    fn from(value: FixedIsize) -> Self {
        value.0 as isize
    }
}

impl From<FixedIsize> for FixedIsizeType {
    #[inline(always)]
    fn from(value: FixedIsize) -> Self {
        value.0
    }
}

impl Formula for FixedIsize {
    const MAX_STACK_SIZE: Option<usize> = Some(size_of::<FixedIsizeType>());
    const EXACT_SIZE: bool = true;
    const HEAPLESS: bool = true;
}

impl BareFormula for FixedIsize {}

impl Serialize<FixedIsize> for FixedIsize {
    #[inline(always)]
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        <FixedIsizeType as Serialize<FixedIsizeType>>::serialize(self.0, sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        Some(Sizes::with_stack(size_of::<FixedIsizeType>()))
    }
}

impl Serialize<FixedIsize> for &FixedIsize {
    #[inline(always)]
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        <FixedIsizeType as Serialize<FixedIsizeType>>::serialize(self.0, sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        Some(Sizes::with_stack(size_of::<FixedIsizeType>()))
    }
}

impl Serialize<FixedIsize> for isize {
    #[inline(always)]
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        Serialize::<FixedIsizeType>::serialize(
            FixedIsize::truncate_unchecked(self).0,
            sizes,
            buffer,
        )
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        Some(Sizes::with_stack(size_of::<FixedIsizeType>()))
    }
}

impl Serialize<FixedIsize> for &isize {
    #[inline(always)]
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        Serialize::<FixedIsizeType>::serialize(
            FixedIsize::truncate_unchecked(*self).0,
            sizes,
            buffer,
        )
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        Some(Sizes::with_stack(size_of::<FixedIsizeType>()))
    }
}

impl Deserialize<'_, FixedIsize> for FixedIsize {
    #[inline(always)]
    fn deserialize(de: Deserializer) -> Result<Self, DeserializeError> {
        let value = <FixedIsizeType as Deserialize<FixedIsizeType>>::deserialize(de)?;

        #[cfg(debug_assertions)]
        if isize::try_from(value).is_err() {
            return Err(DeserializeError::InvalidIsize(value));
        }

        Ok(FixedIsize(value))
    }

    #[inline(always)]
    fn deserialize_in_place(&mut self, de: Deserializer) -> Result<(), DeserializeError> {
        <FixedIsizeType as Deserialize<FixedIsizeType>>::deserialize_in_place(&mut self.0, de)
    }
}

impl Deserialize<'_, FixedIsize> for isize {
    #[inline(always)]
    fn deserialize(de: Deserializer) -> Result<Self, DeserializeError> {
        let value = <FixedIsizeType as Deserialize<FixedIsizeType>>::deserialize(de)?;

        #[cfg(debug_assertions)]
        if isize::try_from(value).is_err() {
            return Err(DeserializeError::InvalidIsize(value));
        }

        Ok(value as isize)
    }

    #[inline(always)]
    fn deserialize_in_place(&mut self, de: Deserializer) -> Result<(), DeserializeError> {
        *self = <Self as Deserialize<FixedIsize>>::deserialize(de)?;
        Ok(())
    }
}
