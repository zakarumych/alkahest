use crate::{
    deserialize::{Deserialize, DeserializeError, Deserializer},
    formula::BareFormula,
};

/// No-op deserializer for any formula.
pub struct Skip;

impl<'de, F> Deserialize<'de, F> for Skip
where
    F: BareFormula + ?Sized,
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
