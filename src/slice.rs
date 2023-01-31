use core::marker::PhantomData;

use crate::{
    schema::SizedSchema, Deserialize, DeserializeError, Deserializer, Schema, Serialize, Serializer,
};

impl<S> Schema for [S] where S: SizedSchema {}

impl<S, T, I> Serialize<[S]> for I
where
    S: SizedSchema,
    I: IntoIterator<Item = T>,
    T: Serialize<S>,
{
    #[inline(always)]
    fn serialize(self, offset: usize, output: &mut [u8]) -> Result<(usize, usize), usize> {
        let mut ser = Serializer::new(offset, output);

        self.into_iter()
            .try_for_each(|elem: T| ser.serialize_value::<S, T>(elem))?;

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
    S: SizedSchema,
    T: Deserialize<'a, S>,
{
    #[inline]
    fn deserialize(len: usize, input: &'a [u8]) -> Result<Self, DeserializeError> {
        if len % S::SIZE != 0 {
            return Err(DeserializeError::WrongLength);
        }
        let count = len / S::SIZE;
        let des = Deserializer::new(input);
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

impl<'a, S, T> Iterator for SliceIter<'a, S, T>
where
    S: SizedSchema,
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

#[cfg(feature = "alloc")]
impl<'a, S, T> Deserialize<'a, [S]> for alloc::vec::Vec<T>
where
    S: SizedSchema,
    T: Deserialize<'a, S>,
{
    fn deserialize(len: usize, input: &'a [u8]) -> Result<Self, DeserializeError> {
        if len % S::SIZE != 0 {
            return Err(DeserializeError::WrongLength);
        }
        let count = len / S::SIZE;
        let mut des = Deserializer::new(input);

        let mut vec = alloc::vec::Vec::with_capacity(count);
        for _ in 0..count {
            vec.push(des.deserialize_sized::<S, T>()?);
        }
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
        let mut des = Deserializer::new(input);

        self.reserve_exact(count);

        for _ in 0..count {
            self.push(des.deserialize_sized::<S, T>()?);
        }

        Ok(())
    }
}
