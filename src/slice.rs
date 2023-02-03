use core::{iter::FusedIterator, marker::PhantomData};

use crate::{
    deserialize::{Deserialize, DeserializeError},
    formula::UnsizedFormula,
    formula::{Formula, NonRefFormula},
    serialize::{Serialize, Serializer},
};

impl<F> UnsizedFormula for [F] where F: Formula {}
impl<F> NonRefFormula for [F] where F: Formula {}

impl<F, T, I> Serialize<[F]> for I
where
    F: Formula,
    I: IntoIterator<Item = T>,
    T: Serialize<F>,
{
    #[inline(always)]
    fn serialize(self, offset: usize, output: &mut [u8]) -> Result<(usize, usize), usize> {
        let mut ser = Serializer::new(offset, output);

        let mut err = None;
        for elem in self.into_iter() {
            match err {
                None => {
                    err = ser.serialize_sized::<F, T>(elem).err();
                }
                Some(size) => {
                    err = Some(size + <T as Serialize<F>>::size(elem));
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
            .fold(0, |acc, elem: T| acc + <T as Serialize<F>>::size(elem))
    }
}

pub struct SliceIter<'de, F, T = F> {
    input: &'de [u8],
    count: usize,
    marker: PhantomData<fn() -> (F, T)>,
}

impl<'de, F, T> Deserialize<'de, [F]> for SliceIter<'de, F, T>
where
    F: Formula,
    T: Deserialize<'de, F>,
{
    #[inline]
    fn deserialize(len: usize, input: &'de [u8]) -> Result<Self, DeserializeError> {
        if len % F::SIZE != 0 {
            return Err(DeserializeError::WrongLength);
        }
        let count = len / F::SIZE;
        Ok(SliceIter {
            input,
            count,
            marker: PhantomData,
        })
    }

    #[inline(always)]
    fn deserialize_in_place(
        &mut self,
        len: usize,
        input: &'de [u8],
    ) -> Result<(), DeserializeError> {
        *self = <Self as Deserialize<[F]>>::deserialize(len, input)?;
        Ok(())
    }
}

impl<'de, F, T, const N: usize> Deserialize<'de, [F; N]> for SliceIter<'de, F, T>
where
    F: Formula,
    T: Deserialize<'de, F>,
{
    #[inline]
    fn deserialize(len: usize, input: &'de [u8]) -> Result<Self, DeserializeError> {
        if len != N * F::SIZE {
            return Err(DeserializeError::WrongLength);
        }
        if input.len() < len {
            return Err(DeserializeError::OutOfBounds);
        }
        Ok(SliceIter {
            input,
            count: N,
            marker: PhantomData,
        })
    }

    #[inline(always)]
    fn deserialize_in_place(
        &mut self,
        len: usize,
        input: &'de [u8],
    ) -> Result<(), DeserializeError> {
        *self = <Self as Deserialize<[F]>>::deserialize(len, input)?;
        Ok(())
    }
}

impl<'de, F, T> Iterator for SliceIter<'de, F, T>
where
    F: Formula,
    T: Deserialize<'de, F>,
{
    type Item = Result<T, DeserializeError>;

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.count, Some(self.count))
    }

    #[inline]
    fn next(&mut self) -> Option<Result<T, DeserializeError>> {
        if self.count == 0 {
            return None;
        }
        let input = self.input;
        self.count -= 1;
        let end = self.input.len() - F::SIZE;
        self.input = &self.input[..end];

        Some(<T as Deserialize<'de, F>>::deserialize(F::SIZE, input))
    }

    #[inline]
    fn count(self) -> usize {
        self.count
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Result<T, DeserializeError>> {
        if n >= self.count {
            self.count = 0;
            return None;
        }
        self.count -= n;
        let end = self.input.len() - F::SIZE * n;
        self.input = &self.input[..end];
        self.next()
    }

    #[inline]
    fn fold<B, Fun>(self, init: B, mut f: Fun) -> B
    where
        Fun: FnMut(B, Result<T, DeserializeError>) -> B,
    {
        let mut accum = init;
        let end = self.input.len();
        for elem in 0..self.count {
            let at = end - F::SIZE * elem;
            let result = <T as Deserialize<'de, F>>::deserialize(F::SIZE, &self.input[..at]);
            accum = f(accum, result);
        }
        accum
    }
}

impl<'de, F, T> DoubleEndedIterator for SliceIter<'de, F, T>
where
    F: Formula,
    T: Deserialize<'de, F>,
{
    #[inline]
    fn next_back(&mut self) -> Option<Result<T, DeserializeError>> {
        if self.count == 0 {
            return None;
        }
        self.count -= 1;
        let at = self.input.len() - F::SIZE * self.count;
        let input = &self.input[at..];

        Some(<T as Deserialize<'de, F>>::deserialize(F::SIZE, input))
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Result<T, DeserializeError>> {
        if n >= self.count {
            self.count = 0;
            return None;
        }
        self.count -= n;
        self.next_back()
    }

    #[inline]
    fn rfold<B, Fun>(self, init: B, mut f: Fun) -> B
    where
        Fun: FnMut(B, Result<T, DeserializeError>) -> B,
    {
        if self.count == 0 {
            return init;
        }
        let start = self.input.len() - F::SIZE * (self.count - 1);
        let mut accum = init;
        for elem in 0..self.count {
            let at = start + F::SIZE * elem;
            let result = <T as Deserialize<'de, F>>::deserialize(F::SIZE, &self.input[..at]);
            accum = f(accum, result);
        }
        accum
    }
}

impl<'de, F, T> ExactSizeIterator for SliceIter<'de, F, T>
where
    F: Formula,
    T: Deserialize<'de, F>,
{
    #[inline]
    fn len(&self) -> usize {
        self.count
    }
}

impl<'de, F, T> FusedIterator for SliceIter<'de, F, T>
where
    F: Formula,
    T: Deserialize<'de, F>,
{
}
