use crate::{
    deserialize::{Deserialize, DeserializeError, Deserializer},
    formula::{Formula, NonRefFormula, UnsizedFormula},
    serialize::{Serialize, Serializer},
};

impl<F, const N: usize> UnsizedFormula for [F; N] where F: Formula {}
impl<F, const N: usize> Formula for [F; N]
where
    F: Formula,
{
    const SIZE: usize = N * F::SIZE;
}
impl<F, const N: usize> NonRefFormula for [F; N] where F: Formula {}

impl<F, T, const N: usize> Serialize<[F; N]> for [T; N]
where
    F: Formula,
    T: Serialize<F>,
{
    #[inline]
    fn serialize(self, offset: usize, output: &mut [u8]) -> Result<(usize, usize), usize> {
        let mut ser = Serializer::new(offset, output);

        let mut err = Ok::<(), usize>(());

        self.into_iter().for_each(|elem: T| {
            if let Err(size) = err {
                err = Err(size + <T as Serialize<F>>::size(elem));
            } else {
                if let Err(size) = ser.serialize_value::<F, T>(elem) {
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
            .fold(0, |acc, elem: T| acc + <T as Serialize<F>>::size(elem))
    }
}

impl<'de, F, T, const N: usize> Serialize<[F; N]> for &'de [T; N]
where
    F: Formula,
    &'de T: Serialize<F>,
{
    #[inline]
    fn serialize(self, offset: usize, output: &mut [u8]) -> Result<(usize, usize), usize> {
        let mut ser = Serializer::new(offset, output);

        let mut err = Ok::<(), usize>(());

        self.iter().for_each(|elem: &'de T| {
            if let Err(size) = err {
                err = Err(size + <&'de T as Serialize<F>>::size(elem));
            } else {
                if let Err(size) = ser.serialize_value::<F, &'de T>(elem) {
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
            .fold(0, |acc, elem: &T| acc + <&T as Serialize<F>>::size(elem))
    }
}

impl<'de, F, T, const N: usize> Deserialize<'de, [F; N]> for [T; N]
where
    F: Formula,
    T: Deserialize<'de, F>,
{
    #[inline(always)]
    fn deserialize(len: usize, input: &'de [u8]) -> Result<Self, DeserializeError> {
        if len != F::SIZE * N {
            return Err(DeserializeError::WrongLength);
        }

        if input.len() < F::SIZE * N {
            return Err(DeserializeError::OutOfBounds);
        }

        let mut des = Deserializer::new(len, input)?;

        let mut opts = [(); N].map(|_| None);
        opts.iter_mut().try_for_each(|slot| {
            *slot = Some(des.deserialize_sized::<F, T>()?);
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
        input: &'de [u8],
    ) -> Result<(), DeserializeError> {
        if len != F::SIZE * N {
            return Err(DeserializeError::WrongLength);
        }

        let mut des = Deserializer::new(len, input)?;
        self.iter_mut()
            .try_for_each(|elem| des.deserialize_in_place_sized::<F, T>(elem))?;

        des.finish_expected();
        Ok(())
    }
}
