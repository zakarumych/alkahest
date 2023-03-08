use core::{mem::size_of, num::TryFromIntError};

use crate::{
    deserialize::{Deserialize, DeserializeError, Deserializer},
    formula::{BareFormula, Formula},
    serialize::{Serialize, Serializer},
};

#[cfg(debug_assertions)]
use crate::cold::err;

#[cfg(feature = "fixed8")]
pub type FixedUsizeType = u8;

#[cfg(feature = "fixed16")]
pub type FixedUsizeType = u16;

#[cfg(feature = "fixed32")]
pub type FixedUsizeType = u32;

#[cfg(feature = "fixed64")]
pub type FixedUsizeType = u64;

#[cfg(feature = "fixed8")]
pub type FixedIsizeType = i8;

#[cfg(feature = "fixed16")]
pub type FixedIsizeType = i16;

#[cfg(feature = "fixed32")]
pub type FixedIsizeType = i32;

#[cfg(feature = "fixed64")]
pub type FixedIsizeType = i64;

/// Type used to represent sizes and offsets in serialized data.
/// This places limitation on sequence sizes which practically is never hit.
/// `usize` itself is not portable and cannot be written into alkahest package.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FixedUsize(FixedUsizeType);

impl FixedUsize {
    #[inline(always)]
    pub fn truncate_unchecked(value: usize) -> Self {
        debug_assert!(FixedUsize::try_from(value).is_ok());
        FixedUsize(value as FixedUsizeType)
    }

    #[inline(always)]
    pub fn to_le_bytes(self) -> [u8; size_of::<Self>()] {
        self.0.to_le_bytes()
    }

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
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        <FixedUsizeType as Serialize<FixedUsizeType>>::serialize(self.0, ser)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<usize> {
        Some(size_of::<FixedUsizeType>())
    }
}

impl Serialize<FixedUsize> for &FixedUsize {
    #[inline(always)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        <FixedUsizeType as Serialize<FixedUsizeType>>::serialize(self.0, ser)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<usize> {
        Some(size_of::<FixedUsizeType>())
    }
}

impl Serialize<FixedUsize> for usize {
    #[inline(always)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Serialize::<FixedUsizeType>::serialize(FixedUsize::truncate_unchecked(self).0, ser)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<usize> {
        Some(size_of::<FixedUsizeType>())
    }
}

impl Serialize<FixedUsize> for &usize {
    #[inline(always)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Serialize::<FixedUsizeType>::serialize(FixedUsize::truncate_unchecked(*self).0, ser)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<usize> {
        Some(size_of::<FixedUsizeType>())
    }
}

impl Deserialize<'_, FixedUsize> for FixedUsize {
    #[inline(always)]
    fn deserialize(de: Deserializer) -> Result<Self, DeserializeError> {
        let value = <FixedUsizeType as Deserialize<FixedUsizeType>>::deserialize(de)?;

        #[cfg(debug_assertions)]
        if usize::try_from(value).is_err() {
            return err(DeserializeError::InvalidUsize(value));
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
            return err(DeserializeError::InvalidUsize(value));
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
    #[inline(always)]
    pub fn truncate_unchecked(value: isize) -> Self {
        debug_assert!(FixedIsize::try_from(value).is_ok());
        FixedIsize(value as FixedIsizeType)
    }

    #[inline(always)]
    pub fn to_le_bytes(self) -> [u8; size_of::<Self>()] {
        self.0.to_le_bytes()
    }

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
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        <FixedIsizeType as Serialize<FixedIsizeType>>::serialize(self.0, ser)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<usize> {
        <FixedIsizeType as Serialize<FixedIsizeType>>::size_hint(&self.0)
    }
}

impl Serialize<FixedIsize> for &FixedIsize {
    #[inline(always)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        <FixedIsizeType as Serialize<FixedIsizeType>>::serialize(self.0, ser)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<usize> {
        <FixedIsizeType as Serialize<FixedIsizeType>>::size_hint(&self.0)
    }
}

impl Serialize<FixedIsize> for isize {
    #[inline(always)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Serialize::<FixedIsizeType>::serialize(FixedIsize::truncate_unchecked(self).0, ser)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<usize> {
        Some(size_of::<FixedIsizeType>())
    }
}

impl Serialize<FixedIsize> for &isize {
    #[inline(always)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Serialize::<FixedIsizeType>::serialize(FixedIsize::truncate_unchecked(*self).0, ser)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<usize> {
        Some(size_of::<FixedIsizeType>())
    }
}

impl Deserialize<'_, FixedIsize> for FixedIsize {
    #[inline(always)]
    fn deserialize(de: Deserializer) -> Result<Self, DeserializeError> {
        let value = <FixedIsizeType as Deserialize<FixedIsizeType>>::deserialize(de)?;

        #[cfg(debug_assertions)]
        if isize::try_from(value).is_err() {
            return err(DeserializeError::InvalidIsize(value));
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
            return err(DeserializeError::InvalidIsize(value));
        }

        Ok(value as isize)
    }

    #[inline(always)]
    fn deserialize_in_place(&mut self, de: Deserializer) -> Result<(), DeserializeError> {
        *self = <Self as Deserialize<FixedIsize>>::deserialize(de)?;
        Ok(())
    }
}
