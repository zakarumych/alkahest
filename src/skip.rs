use crate::{
    deserialize::{Deserialize, Deserializer, Error},
    formula::BareFormula,
};

/// No-op deserializer for any formula.
pub struct Skip;

impl<'de, F> Deserialize<'de, F> for Skip
where
    F: BareFormula + ?Sized,
{
    #[inline(never)]
    fn deserialize(_de: Deserializer) -> Result<Self, Error> {
        Ok(Skip)
    }

    #[inline(never)]
    fn deserialize_in_place(&mut self, _de: Deserializer) -> Result<(), Error> {
        Ok(())
    }
}
