use core::{mem::size_of, num::TryFromIntError};

use crate::{
    cold::err,
    deserialize::{Deserialize, Deserializer, Error},
    formula::{BareFormula, Formula},
    serialize::{Serialize, Serializer},
};

#[cfg(feature = "fixed8")]
pub type FixedUsizeType = u32;

#[cfg(feature = "fixed16")]
pub type FixedUsizeType = u64;

#[cfg(feature = "fixed32")]
pub type FixedUsizeType = u32;

#[cfg(feature = "fixed64")]
pub type FixedUsizeType = u64;

#[cfg(feature = "fixed8")]
pub type FixedIsizeType = i32;

#[cfg(feature = "fixed16")]
pub type FixedIsizeType = i64;

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
    #[inline(never)]
    pub fn truncate_unchecked(value: usize) -> Self {
        debug_assert!(FixedUsize::try_from(value).is_ok());
        FixedUsize(value as FixedUsizeType)
    }

    #[inline(never)]
    pub fn to_le_bytes(self) -> [u8; size_of::<Self>()] {
        self.0.to_le_bytes()
    }

    #[inline(never)]
    pub fn from_le_bytes(bytes: [u8; size_of::<Self>()]) -> Result<Self, TryFromIntError> {
        FixedUsizeType::from_le_bytes(bytes).try_into()
    }
}

impl TryFrom<usize> for FixedUsize {
    type Error = TryFromIntError;

    #[inline(never)]
    fn try_from(value: usize) -> Result<Self, TryFromIntError> {
        FixedUsizeType::try_from(value).map(FixedUsize)
    }
}

impl TryFrom<FixedUsizeType> for FixedUsize {
    type Error = TryFromIntError;

    #[inline(never)]
    fn try_from(value: FixedUsizeType) -> Result<Self, TryFromIntError> {
        usize::try_from(value)?;
        Ok(FixedUsize(value))
    }
}

impl From<FixedUsize> for usize {
    #[inline(never)]
    fn from(value: FixedUsize) -> Self {
        value.0 as usize
    }
}

impl From<FixedUsize> for FixedUsizeType {
    #[inline(never)]
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
    #[inline(never)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        <FixedUsizeType as Serialize<FixedUsizeType>>::serialize(self.0, ser)
    }

    #[inline(never)]
    fn size_hint(&self) -> Option<usize> {
        Some(size_of::<FixedUsizeType>())
    }
}

impl Serialize<FixedUsize> for &FixedUsize {
    #[inline(never)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        <FixedUsizeType as Serialize<FixedUsizeType>>::serialize(self.0, ser)
    }

    #[inline(never)]
    fn size_hint(&self) -> Option<usize> {
        Some(size_of::<FixedUsizeType>())
    }
}

impl Serialize<FixedUsize> for usize {
    #[inline(never)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Serialize::<FixedUsizeType>::serialize(FixedUsize::truncate_unchecked(self).0, ser)
    }

    #[inline(never)]
    fn size_hint(&self) -> Option<usize> {
        Some(size_of::<FixedUsizeType>())
    }
}

impl Serialize<FixedUsize> for &usize {
    #[inline(never)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Serialize::<FixedUsizeType>::serialize(FixedUsize::truncate_unchecked(*self).0, ser)
    }

    #[inline(never)]
    fn size_hint(&self) -> Option<usize> {
        Some(size_of::<FixedUsizeType>())
    }
}

impl Deserialize<'_, FixedUsize> for FixedUsize {
    #[inline(never)]
    fn deserialize(de: Deserializer) -> Result<Self, Error> {
        let value = <FixedUsizeType as Deserialize<FixedUsizeType>>::deserialize(de)?;

        #[cfg(debug_assertions)]
        if usize::try_from(value).is_err() {
            return err(Error::InvalidUsize(value));
        }

        Ok(FixedUsize(value))
    }

    #[inline(never)]
    fn deserialize_in_place(&mut self, de: Deserializer) -> Result<(), Error> {
        <FixedUsizeType as Deserialize<FixedUsizeType>>::deserialize_in_place(&mut self.0, de)
    }
}

impl Deserialize<'_, FixedUsize> for usize {
    #[inline(never)]
    fn deserialize(de: Deserializer) -> Result<Self, Error> {
        let value = <FixedUsizeType as Deserialize<FixedUsizeType>>::deserialize(de)?;

        #[cfg(debug_assertions)]
        if usize::try_from(value).is_err() {
            return err(Error::InvalidUsize(value));
        }

        Ok(value as usize)
    }

    #[inline(never)]
    fn deserialize_in_place(&mut self, de: Deserializer) -> Result<(), Error> {
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
    #[inline(never)]
    pub fn truncate_unchecked(value: isize) -> Self {
        debug_assert!(FixedIsize::try_from(value).is_ok());
        FixedIsize(value as FixedIsizeType)
    }

    #[inline(never)]
    pub fn to_le_bytes(self) -> [u8; size_of::<Self>()] {
        self.0.to_le_bytes()
    }

    #[inline(never)]
    pub fn from_le_bytes(bytes: [u8; size_of::<Self>()]) -> Result<Self, TryFromIntError> {
        FixedIsizeType::from_le_bytes(bytes).try_into()
    }
}

impl TryFrom<isize> for FixedIsize {
    type Error = TryFromIntError;

    #[inline(never)]
    fn try_from(value: isize) -> Result<Self, TryFromIntError> {
        FixedIsizeType::try_from(value).map(FixedIsize)
    }
}

impl TryFrom<FixedIsizeType> for FixedIsize {
    type Error = TryFromIntError;

    #[inline(never)]
    fn try_from(value: FixedIsizeType) -> Result<Self, TryFromIntError> {
        isize::try_from(value)?;
        Ok(FixedIsize(value))
    }
}

impl From<FixedIsize> for isize {
    #[inline(never)]
    fn from(value: FixedIsize) -> Self {
        value.0 as isize
    }
}

impl From<FixedIsize> for FixedIsizeType {
    #[inline(never)]
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
    #[inline(never)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        <FixedIsizeType as Serialize<FixedIsizeType>>::serialize(self.0, ser)
    }

    #[inline(never)]
    fn size_hint(&self) -> Option<usize> {
        <FixedIsizeType as Serialize<FixedIsizeType>>::size_hint(&self.0)
    }
}

impl Serialize<FixedIsize> for &FixedIsize {
    #[inline(never)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        <FixedIsizeType as Serialize<FixedIsizeType>>::serialize(self.0, ser)
    }

    #[inline(never)]
    fn size_hint(&self) -> Option<usize> {
        <FixedIsizeType as Serialize<FixedIsizeType>>::size_hint(&self.0)
    }
}

impl Serialize<FixedIsize> for isize {
    #[inline(never)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Serialize::<FixedIsizeType>::serialize(FixedIsize::truncate_unchecked(self).0, ser)
    }

    #[inline(never)]
    fn size_hint(&self) -> Option<usize> {
        Some(size_of::<FixedIsizeType>())
    }
}

impl Serialize<FixedIsize> for &isize {
    #[inline(never)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Serialize::<FixedIsizeType>::serialize(FixedIsize::truncate_unchecked(*self).0, ser)
    }

    #[inline(never)]
    fn size_hint(&self) -> Option<usize> {
        Some(size_of::<FixedIsizeType>())
    }
}

impl Deserialize<'_, FixedIsize> for FixedIsize {
    #[inline(never)]
    fn deserialize(de: Deserializer) -> Result<Self, Error> {
        let value = <FixedIsizeType as Deserialize<FixedIsizeType>>::deserialize(de)?;

        #[cfg(debug_assertions)]
        if isize::try_from(value).is_err() {
            return err(Error::InvalidIsize(value));
        }

        Ok(FixedIsize(value))
    }

    #[inline(never)]
    fn deserialize_in_place(&mut self, de: Deserializer) -> Result<(), Error> {
        <FixedIsizeType as Deserialize<FixedIsizeType>>::deserialize_in_place(&mut self.0, de)
    }
}

impl Deserialize<'_, FixedIsize> for isize {
    #[inline(never)]
    fn deserialize(de: Deserializer) -> Result<Self, Error> {
        let value = <FixedIsizeType as Deserialize<FixedIsizeType>>::deserialize(de)?;

        #[cfg(debug_assertions)]
        if isize::try_from(value).is_err() {
            return err(Error::InvalidIsize(value));
        }

        Ok(value as isize)
    }

    #[inline(never)]
    fn deserialize_in_place(&mut self, de: Deserializer) -> Result<(), Error> {
        *self = <Self as Deserialize<FixedIsize>>::deserialize(de)?;
        Ok(())
    }
}
