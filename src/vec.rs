use alloc::vec::Vec;

use crate::{
    deserialize::{Deserialize, DeserializeError, Deserializer},
    formula::{Formula, FormulaAlias},
    reference::Ref,
};

impl<S> FormulaAlias for Vec<S>
where
    S: Formula,
{
    type Alias = Ref<[S]>;
}

impl<'a, S, T> Deserialize<'a, [S]> for Vec<T>
where
    S: Formula,
    T: Deserialize<'a, S>,
{
    fn deserialize(len: usize, input: &'a [u8]) -> Result<Self, DeserializeError> {
        if len % S::SIZE != 0 {
            return Err(DeserializeError::WrongLength);
        }
        let count = len / S::SIZE;
        let mut des = Deserializer::new(len, input);

        let mut vec = Vec::with_capacity(count);
        for _ in 0..count {
            vec.push(des.deserialize_sized::<S, T>()?);
        }

        des.finish_expected();
        Ok(vec)
    }

    fn deserialize_in_place(
        &mut self,
        len: usize,
        input: &'a [u8],
    ) -> Result<(), DeserializeError> {
        if len % S::SIZE != 0 {
            return Err(DeserializeError::WrongLength);
        }
        let count = len / S::SIZE;
        let mut des = Deserializer::new(len, input);

        self.reserve_exact(count);

        for _ in 0..count {
            self.push(des.deserialize_sized::<S, T>()?);
        }
        des.finish_expected();

        Ok(())
    }
}

impl<'a, S, T, const N: usize> Deserialize<'a, [S; N]> for Vec<T>
where
    S: Formula,
    T: Deserialize<'a, S>,
{
    fn deserialize(len: usize, input: &'a [u8]) -> Result<Self, DeserializeError> {
        if len != N * S::SIZE {
            return Err(DeserializeError::WrongLength);
        }
        let mut des = Deserializer::new(len, input);

        let mut vec = Vec::with_capacity(N);
        for _ in 0..N {
            vec.push(des.deserialize_sized::<S, T>()?);
        }

        des.finish_expected();
        Ok(vec)
    }

    fn deserialize_in_place(
        &mut self,
        len: usize,
        input: &'a [u8],
    ) -> Result<(), DeserializeError> {
        if len != N * S::SIZE {
            return Err(DeserializeError::WrongLength);
        }
        let mut des = Deserializer::new(len, input);

        self.reserve_exact(N);

        for _ in 0..N {
            self.push(des.deserialize_sized::<S, T>()?);
        }
        des.finish_expected();

        Ok(())
    }
}
