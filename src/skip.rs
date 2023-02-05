use crate::{
    deserialize::{Deserialize, Deserializer, Error},
    formula::NonRefFormula,
};

/// No-op deserializer for any formula.
pub struct Skip;

impl<'de, F> Deserialize<'de, F> for Skip
where
    F: NonRefFormula + ?Sized,
{
    #[inline(always)]
    fn deserialize(_de: Deserializer) -> Result<Self, Error> {
        Ok(Skip)
    }

    #[inline(always)]
    fn deserialize_in_place(&mut self, _de: Deserializer) -> Result<(), Error> {
        Ok(())
    }
}
