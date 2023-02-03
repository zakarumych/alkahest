use alloc::vec::Vec;

use crate::{
    bytes::Bytes,
    deserialize::{Deserialize, DeserializeError, Deserializer},
    formula::{Formula, UnsizedFormula},
    reference::Ref,
    serialize::Serialize,
};

impl<F> UnsizedFormula for Vec<F> where F: Formula {}
impl<F> Formula for Vec<F>
where
    F: Formula,
{
    const SIZE: usize = <Ref<[F]> as Formula>::SIZE;
}

impl<F, T, I> Serialize<Vec<F>> for I
where
    F: Formula,
    I: IntoIterator<Item = T>,
    T: Serialize<F>,
{
    fn serialize(self, offset: usize, output: &mut [u8]) -> Result<(usize, usize), usize> {
        <I as Serialize<Ref<[F]>>>::serialize(self, offset, output)
    }

    fn size(self) -> usize {
        <I as Serialize<Ref<[F]>>>::size(self)
    }
}

impl<'de, F, T> Deserialize<'de, Vec<F>> for T
where
    F: Formula,
    T: Deserialize<'de, Ref<[F]>> + ?Sized,
{
    fn deserialize(len: usize, input: &'de [u8]) -> Result<Self, DeserializeError>
    where
        T: Sized,
    {
        <T as Deserialize<'de, Ref<[F]>>>::deserialize(len, input)
    }

    fn deserialize_in_place(
        &mut self,
        len: usize,
        input: &'de [u8],
    ) -> Result<(), DeserializeError> {
        <T as Deserialize<'de, Ref<[F]>>>::deserialize_in_place(self, len, input)
    }
}

impl<'de, F, T> Deserialize<'de, [F]> for Vec<T>
where
    F: Formula,
    T: Deserialize<'de, F>,
{
    fn deserialize(len: usize, input: &'de [u8]) -> Result<Self, DeserializeError> {
        if len % F::SIZE != 0 {
            return Err(DeserializeError::WrongLength);
        }
        let mut des = Deserializer::new(len, input)?;

        let count = len / F::SIZE;
        let mut vec = Vec::with_capacity(count);
        for _ in 0..count {
            vec.push(des.deserialize_sized::<F, T>()?);
        }

        des.finish_expected();
        Ok(vec)
    }

    fn deserialize_in_place(
        &mut self,
        len: usize,
        input: &'de [u8],
    ) -> Result<(), DeserializeError> {
        if len % F::SIZE != 0 {
            return Err(DeserializeError::WrongLength);
        }
        let mut des = Deserializer::new(len, input)?;

        let count = len / F::SIZE;
        self.reserve_exact(count);
        for _ in 0..count {
            self.push(des.deserialize_sized::<F, T>()?);
        }
        des.finish_expected();

        Ok(())
    }
}

impl<'de, F, T, const N: usize> Deserialize<'de, [F; N]> for Vec<T>
where
    F: Formula,
    T: Deserialize<'de, F>,
{
    fn deserialize(len: usize, input: &'de [u8]) -> Result<Self, DeserializeError> {
        if len != N * F::SIZE {
            return Err(DeserializeError::WrongLength);
        }

        let mut des = Deserializer::new(len, input)?;

        let mut vec = Vec::with_capacity(N);
        for _ in 0..N {
            vec.push(des.deserialize_sized::<F, T>()?);
        }

        des.finish_expected();
        Ok(vec)
    }

    fn deserialize_in_place(
        &mut self,
        len: usize,
        input: &'de [u8],
    ) -> Result<(), DeserializeError> {
        if len != N * F::SIZE {
            return Err(DeserializeError::WrongLength);
        }
        let mut des = Deserializer::new(len, input)?;

        self.reserve_exact(N);
        for _ in 0..N {
            self.push(des.deserialize_sized::<F, T>()?);
        }

        des.finish_expected();
        Ok(())
    }
}

impl<'de> Deserialize<'de, Bytes> for Vec<u8> {
    #[inline(always)]
    fn deserialize(len: usize, input: &'de [u8]) -> Result<Self, DeserializeError> {
        if len > input.len() {
            return Err(DeserializeError::OutOfBounds);
        }
        let at = input.len() - len;
        Ok(input[at..].to_vec())
    }

    #[inline(always)]
    fn deserialize_in_place(
        &mut self,
        len: usize,
        input: &'de [u8],
    ) -> Result<(), DeserializeError> {
        if len > input.len() {
            return Err(DeserializeError::OutOfBounds);
        }
        let at = input.len() - len;
        self.extend_from_slice(&input[at..]);
        Ok(())
    }
}
