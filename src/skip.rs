use crate::{
    deserialize::{Deserialize, DeserializeError},
    formula::NonRefFormula,
};

/// No-op deserializer for any formula.
pub struct Skip;

impl<'de, F> Deserialize<'de, F> for Skip
where
    F: NonRefFormula + ?Sized,
{
    fn deserialize(_len: usize, _input: &[u8]) -> Result<Self, DeserializeError> {
        Ok(Skip)
    }

    fn deserialize_in_place(
        &mut self,
        _len: usize,
        _input: &'de [u8],
    ) -> Result<(), DeserializeError> {
        Ok(())
    }
}
