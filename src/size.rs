use core::{mem::size_of, num::TryFromIntError};

use crate::{
    deserialize::{Deserialize, DeserializeError},
    formula::{Formula, UnsizedFormula},
    serialize::Serialize,
};

pub type FixedUsizeType = u32;

/// Type used to represent sizes and offsets in serialized data.
/// This places limitation on sequence sizes which practically is never hit.
/// `usize` itself is not portable and cannot be written into alkahest package.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FixedUsize(FixedUsizeType);

impl FixedUsize {
    #[inline(always)]
    pub fn truncated(value: usize) -> Self {
        FixedUsize(value as FixedUsizeType)
    }

    #[inline(always)]
    pub fn to_le_bytes(self) -> [u8; size_of::<Self>()] {
        self.0.to_le_bytes()
    }

    #[inline(always)]
    pub fn from_le_bytes(bytes: [u8; size_of::<Self>()]) -> Self {
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

impl UnsizedFormula for FixedUsize {}
impl Formula for FixedUsize {
    const SIZE: usize = size_of::<FixedUsizeType>();
}

impl Serialize<FixedUsize> for FixedUsize {
    #[inline(always)]
    fn serialize(self, offset: usize, output: &mut [u8]) -> Result<(usize, usize), usize> {
        <FixedUsizeType as Serialize<FixedUsizeType>>::serialize(self.0, offset, output)
    }
}

impl Serialize<FixedUsize> for &'_ FixedUsize {
    #[inline(always)]
    fn serialize(self, offset: usize, output: &mut [u8]) -> Result<(usize, usize), usize> {
        <FixedUsizeType as Serialize<FixedUsizeType>>::serialize(self.0, offset, output)
    }
}

impl Deserialize<'_, FixedUsize> for FixedUsize {
    #[inline(always)]
    fn deserialize(len: usize, input: &[u8]) -> Result<Self, DeserializeError> {
        let value = <FixedUsizeType as Deserialize<FixedUsizeType>>::deserialize(len, input)?;
        if value > usize::MAX as FixedUsizeType {
            return Err(DeserializeError::InvalidSize(value));
        }

        Ok(FixedUsize(value))
    }

    #[inline(always)]
    fn deserialize_in_place(&mut self, len: usize, input: &[u8]) -> Result<(), DeserializeError> {
        <FixedUsizeType as Deserialize<FixedUsizeType>>::deserialize_in_place(
            &mut self.0,
            len,
            input,
        )
    }
}
