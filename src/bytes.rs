use crate::{
    deserialize::{Deserialize, DeserializeError},
    formula::{NonRefFormula, UnsizedFormula},
    serialize::Serialize,
};

/// A formula for a raw byte slices.
/// Serializable from anything that implements `AsRef<[u8]>`.
pub struct Bytes;

impl UnsizedFormula for Bytes {}
impl NonRefFormula for Bytes {}

impl<T> Serialize<Bytes> for T
where
    T: AsRef<[u8]>,
{
    #[inline(always)]
    fn serialize(self, _offset: usize, output: &mut [u8]) -> Result<(usize, usize), usize> {
        let slice = self.as_ref();
        if slice.len() > output.len() {
            return Err(slice.len());
        }
        let at = output.len() - slice.len();
        output[at..].copy_from_slice(slice);
        Ok((0, slice.len()))
    }

    #[inline(always)]
    fn size(self) -> usize {
        self.as_ref().len()
    }
}

impl<'de> Deserialize<'de, Bytes> for &'de [u8] {
    #[inline(always)]
    fn deserialize(len: usize, input: &'de [u8]) -> Result<Self, DeserializeError> {
        if len > input.len() {
            return Err(DeserializeError::OutOfBounds);
        }
        Ok(&input[input.len() - len..])
    }

    #[inline(always)]
    fn deserialize_in_place(
        &mut self,
        len: usize,
        input: &'de [u8],
    ) -> Result<(), DeserializeError> {
        *self = <Self as Deserialize<'de, Bytes>>::deserialize(len, input)?;
        Ok(())
    }
}
