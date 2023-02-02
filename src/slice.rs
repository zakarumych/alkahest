use core::marker::PhantomData;

use crate::{
    formula::Formula, Deserialize, DeserializeError, Deserializer, Serialize, Serializer,
    UnsizedFormula,
};

impl<S> UnsizedFormula for [S] where S: Formula {}

impl<S, T, I> Serialize<[S]> for I
where
    S: Formula,
    I: IntoIterator<Item = T>,
    T: Serialize<S>,
{
    #[inline(always)]
    fn serialize(self, offset: usize, output: &mut [u8]) -> Result<(usize, usize), usize> {
        let mut ser = Serializer::new(offset, output);

        let mut err = None;
        for elem in self.into_iter() {
            match err {
                None => {
                    err = ser.serialize_value::<S, T>(elem).err();
                }
                Some(size) => {
                    err = Some(size + <T as Serialize<S>>::size(elem));
                }
            }
        }

        if let Some(size) = err {
            return Err(size);
        }

        Ok(ser.finish())
    }

    #[inline]
    fn size(self) -> usize {
        self.into_iter()
            .fold(0, |acc, elem: T| acc + <T as Serialize<S>>::size(elem))
    }
}

pub struct SliceIter<'a, S, T = S> {
    des: Deserializer<'a>,
    count: usize,
    marker: PhantomData<fn() -> (S, T)>,
}

impl<'a, S, T> Deserialize<'a, [S]> for SliceIter<'a, S, T>
where
    S: Formula,
    T: Deserialize<'a, S>,
{
    #[inline]
    fn deserialize(len: usize, input: &'a [u8]) -> Result<Self, DeserializeError> {
        if len % S::SIZE != 0 {
            return Err(DeserializeError::WrongLength);
        }
        let count = len / S::SIZE;
        let des = Deserializer::new(len, input);
        Ok(SliceIter {
            des,
            count,
            marker: PhantomData,
        })
    }

    #[inline(always)]
    fn deserialize_in_place(
        &mut self,
        len: usize,
        input: &'a [u8],
    ) -> Result<(), DeserializeError> {
        *self = <Self as Deserialize<[S]>>::deserialize(len, input)?;
        Ok(())
    }
}

impl<'a, S, T, const N: usize> Deserialize<'a, [S; N]> for SliceIter<'a, S, T>
where
    S: Formula,
    T: Deserialize<'a, S>,
{
    #[inline]
    fn deserialize(len: usize, input: &'a [u8]) -> Result<Self, DeserializeError> {
        if len != N * S::SIZE {
            return Err(DeserializeError::WrongLength);
        }
        let des = Deserializer::new(len, input);
        Ok(SliceIter {
            des,
            count: N,
            marker: PhantomData,
        })
    }

    #[inline(always)]
    fn deserialize_in_place(
        &mut self,
        len: usize,
        input: &'a [u8],
    ) -> Result<(), DeserializeError> {
        *self = <Self as Deserialize<[S]>>::deserialize(len, input)?;
        Ok(())
    }
}

impl<'a, S, T> Iterator for SliceIter<'a, S, T>
where
    S: Formula,
    T: Deserialize<'a, S>,
{
    type Item = Result<T, DeserializeError>;

    #[inline]
    fn next(&mut self) -> Option<Result<T, DeserializeError>> {
        if self.count == 0 {
            return None;
        }

        match self.des.deserialize_sized::<S, T>() {
            Ok(value) => {
                self.count -= 1;
                Some(Ok(value))
            }
            Err(err) => Some(Err(err)),
        }
    }
}
