use crate::{
    deserialize::{Deserialize, Deserializer, Error},
    formula::{repeat_size, Formula, NonRefFormula},
    serialize::{SerializeOwned, Serializer},
};

impl<F, const N: usize> NonRefFormula for [F; N]
where
    F: Formula,
{
    const MAX_SIZE: Option<usize> = repeat_size(F::MAX_SIZE, N);
}

impl<F, T, const N: usize> SerializeOwned<[F; N]> for [T; N]
where
    F: NonRefFormula,
    T: SerializeOwned<F>,
{
    #[cfg_attr(feature = "inline-more", inline(always))]
    fn serialize_owned<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut ser = ser.into();
        self.into_iter()
            .try_for_each(|elem: T| ser.write_value::<F, T>(elem))?;
        ser.finish()
    }
}

impl<F, T, const N: usize> SerializeOwned<[F; N]> for &[T; N]
where
    F: NonRefFormula,
    for<'ser> &'ser T: SerializeOwned<F>,
{
    #[cfg_attr(feature = "inline-more", inline(always))]
    fn serialize_owned<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut ser = ser.into();
        self.iter()
            .try_for_each(|elem: &T| ser.write_value::<F, &T>(elem))?;
        ser.finish()
    }
}

impl<'de, F, T, const N: usize> Deserialize<'de, [F; N]> for [T; N]
where
    F: NonRefFormula,
    T: Deserialize<'de, F>,
{
    #[cfg_attr(feature = "inline-more", inline(always))]
    fn deserialize(mut de: Deserializer<'de>) -> Result<Self, Error> {
        let mut opts = [(); N].map(|_| None);
        opts.iter_mut().try_for_each(|slot| {
            *slot = Some(de.read_value::<F, T>()?);
            Ok(())
        })?;
        let value = opts.map(|slot| slot.unwrap());
        Ok(value)
    }

    #[cfg_attr(feature = "inline-more", inline(always))]
    fn deserialize_in_place(&mut self, mut de: Deserializer<'de>) -> Result<(), Error> {
        self.iter_mut()
            .try_for_each(|elem| de.read_in_place::<F, T>(elem))?;
        Ok(())
    }
}
