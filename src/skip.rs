use crate::{
    deserialize::{Deserialize, DeserializeError, Deserializer},
    formula::BareFormulaType,
};

/// No-op deserializer for any formula.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Skip;

impl<'de, F> Deserialize<'de, F> for Skip
where
    F: BareFormulaType + ?Sized,
{
    #[inline(always)]
    fn deserialize(_de: Deserializer) -> Result<Self, DeserializeError> {
        Ok(Skip)
    }

    #[inline(always)]
    fn deserialize_in_place(&mut self, _de: Deserializer) -> Result<(), DeserializeError> {
        Ok(())
    }
}
