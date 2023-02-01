use crate::{
    schema::{Schema, SizedSchema},
    serialize::Serialize,
    Deserialize, DeserializeError, Deserializer, Serializer,
};

impl<S, const N: usize> Schema for [S; N] where S: SizedSchema {}
impl<S, const N: usize> SizedSchema for [S; N]
where
    S: SizedSchema,
{
    const SIZE: usize = N * S::SIZE;
}

impl<S, T, const N: usize> Serialize<[S; N]> for [T; N]
where
    S: SizedSchema,
    T: Serialize<S>,
{
    #[inline]
    fn serialize(self, offset: usize, output: &mut [u8]) -> Result<(usize, usize), usize> {
        let mut ser = Serializer::new(offset, output);

        let mut err = Ok::<(), usize>(());

        self.into_iter().for_each(|elem: T| {
            if let Err(size) = err {
                err = Err(size + <T as Serialize<S>>::size(elem));
            } else {
                if let Err(size) = ser.serialize_value::<S, T>(elem) {
                    err = Err(size);
                }
            }
        });

        err?;
        Ok(ser.finish())
    }

    #[inline]
    fn size(self) -> usize {
        self.into_iter()
            .fold(0, |acc, elem: T| acc + <T as Serialize<S>>::size(elem))
    }
}

impl<'a, S, T, const N: usize> Serialize<[S; N]> for &'a [T; N]
where
    S: SizedSchema,
    &'a T: Serialize<S>,
{
    #[inline]
    fn serialize(self, offset: usize, output: &mut [u8]) -> Result<(usize, usize), usize> {
        let mut ser = Serializer::new(offset, output);

        let mut err = Ok::<(), usize>(());

        self.iter().for_each(|elem: &'a T| {
            if let Err(size) = err {
                err = Err(size + <&'a T as Serialize<S>>::size(elem));
            } else {
                if let Err(size) = ser.serialize_value::<S, &'a T>(elem) {
                    err = Err(size);
                }
            }
        });

        err?;
        Ok(ser.finish())
    }

    #[inline]
    fn size(self) -> usize {
        self.into_iter()
            .fold(0, |acc, elem: &T| acc + <&T as Serialize<S>>::size(elem))
    }
}

impl<'a, S, T, const N: usize> Deserialize<'a, [S; N]> for [T; N]
where
    S: SizedSchema,
    T: Deserialize<'a, S>,
{
    #[inline(always)]
    fn deserialize(len: usize, input: &'a [u8]) -> Result<Self, DeserializeError> {
        if len != S::SIZE * N {
            return Err(DeserializeError::WrongLength);
        }

        if input.len() < S::SIZE * N {
            return Err(DeserializeError::OutOfBounds);
        }

        let mut des = Deserializer::new(len, input);

        let mut opts = [(); N].map(|_| None);
        opts.iter_mut().try_for_each(|slot| {
            *slot = Some(des.deserialize_sized::<S, T>()?);
            Ok(())
        })?;

        let value = opts.map(|slot| slot.unwrap());
        des.finish_expected();
        Ok(value)
    }

    #[inline(always)]
    fn deserialize_in_place(
        &mut self,
        len: usize,
        input: &'a [u8],
    ) -> Result<(), DeserializeError> {
        if len != S::SIZE * N {
            return Err(DeserializeError::WrongLength);
        }

        if input.len() < S::SIZE * N {
            return Err(DeserializeError::OutOfBounds);
        }

        let mut des = Deserializer::new(len, input);
        self.iter_mut()
            .try_for_each(|elem| des.deserialize_in_place_sized::<S, T>(elem))?;
        des.finish_expected();

        Ok(())
    }
}

trait MapArrayRef<const N: usize> {
    type Item: Sized;

    fn map_ref<'a, F, U>(&'a self, f: F) -> [U; N]
    where
        F: FnMut(&'a Self::Item) -> U;

    fn map_mut<'a, F, U>(&'a mut self, f: F) -> [U; N]
    where
        F: FnMut(&'a mut Self::Item) -> U;
}

impl<T, const N: usize> MapArrayRef<N> for [T; N] {
    type Item = T;

    #[inline]
    fn map_ref<'a, F, U>(&'a self, mut f: F) -> [U; N]
    where
        F: FnMut(&'a Self::Item) -> U,
    {
        let mut iter = self.iter();
        [(); N].map(|()| f(iter.next().unwrap()))
    }

    #[inline]
    fn map_mut<'a, F, U>(&'a mut self, mut f: F) -> [U; N]
    where
        F: FnMut(&'a mut Self::Item) -> U,
    {
        let mut iter = self.iter_mut();
        [(); N].map(|()| f(iter.next().unwrap()))
    }
}
