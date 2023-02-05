use crate::{
    deserialize::{Deserialize, Deserializer, Error},
    formula::{repeat_size, Formula, NonRefFormula},
    serialize::{Serialize, Serializer},
};

impl<F, const N: usize> Formula for [F; N]
where
    F: Formula,
{
    const MAX_SIZE: Option<usize> = repeat_size(F::MAX_SIZE, N);
}
impl<F, const N: usize> NonRefFormula for [F; N] where F: Formula {}

impl<F, T, const N: usize> Serialize<[F; N]> for [T; N]
where
    F: Formula,
    T: Serialize<F>,
{
    #[inline(always)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut ser = ser.into();
        self.into_iter()
            .try_for_each(|elem: T| ser.write_value::<F, T>(elem))?;
        ser.finish()
    }
}

impl<'ser, F, T, const N: usize> Serialize<[F; N]> for &'ser [T; N]
where
    F: Formula,
    &'ser T: Serialize<F>,
{
    #[inline(always)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut ser = ser.into();
        self.iter()
            .try_for_each(|elem: &'ser T| ser.write_value::<F, &'ser T>(elem))?;
        ser.finish()
    }
}

impl<'de, F, T, const N: usize> Deserialize<'de, [F; N]> for [T; N]
where
    F: Formula,
    T: Deserialize<'de, F>,
{
    #[inline(always)]
    fn deserialize(mut de: Deserializer<'de>) -> Result<Self, Error> {
        let mut opts = [(); N].map(|_| None);
        opts.iter_mut().try_for_each(|slot| {
            *slot = Some(de.read_value::<F, T>()?);
            Ok(())
        })?;
        let value = opts.map(|slot| slot.unwrap());
        Ok(value)
    }

    #[inline(always)]
    fn deserialize_in_place(&mut self, mut de: Deserializer<'de>) -> Result<(), Error> {
        self.iter_mut()
            .try_for_each(|elem| de.read_in_place::<F, T>(elem))?;
        Ok(())
    }
}
