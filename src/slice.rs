use crate::{
    deserialize::{DeIter, Deserialize, Deserializer, Error},
    formula::{Formula, NonRefFormula},
    serialize::{Serialize, Serializer},
};

impl<F> Formula for [F]
where
    F: Formula,
{
    const MAX_STACK_SIZE: Option<usize> = None;
    const EXACT_SIZE: bool = F::EXACT_SIZE;
}

impl<F> NonRefFormula for [F] where F: Formula {}

impl<F, T, I> Serialize<[F]> for I
where
    F: Formula,
    I: IntoIterator<Item = T>,
    T: Serialize<F>,
{
    #[inline(always)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut ser = ser.into();
        for elem in self {
            ser.write_value::<F, T>(elem)?;
        }
        ser.finish()
    }
}

pub struct LazySlice<'de, F: ?Sized, T = F> {
    inner: DeIter<'de, F, T>,
}

impl<'de, F, T> LazySlice<'de, F, T>
where
    F: ?Sized,
{
    #[inline(always)]
    pub fn iter(&self) -> DeIter<'de, F, T>
    where
        F: Formula,
        T: Deserialize<'de, F>,
    {
        self.inner.clone()
    }
}

impl<'de, F, T> IntoIterator for LazySlice<'de, F, T>
where
    F: Formula + ?Sized,
    T: Deserialize<'de, F>,
{
    type Item = Result<T, Error>;
    type IntoIter = DeIter<'de, F, T>;

    #[inline(always)]
    fn into_iter(self) -> DeIter<'de, F, T> {
        self.inner
    }
}

impl<'de, 'fe: 'de, F, T> Deserialize<'fe, [F]> for LazySlice<'de, F, T>
where
    F: Formula,
    T: Deserialize<'de, F>,
{
    #[inline(always)]
    fn deserialize(de: Deserializer<'fe>) -> Result<Self, Error> {
        Ok(LazySlice {
            inner: de.into_iter()?,
        })
    }

    #[inline(always)]
    fn deserialize_in_place(&mut self, de: Deserializer<'fe>) -> Result<(), Error> {
        self.inner = de.into_iter()?;
        Ok(())
    }
}

impl<'de, 'fe: 'de, F, T, const N: usize> Deserialize<'fe, [F; N]> for LazySlice<'de, F, T>
where
    F: Formula,
    T: Deserialize<'de, F>,
{
    #[inline(always)]
    fn deserialize(de: Deserializer<'fe>) -> Result<Self, Error> {
        Ok(LazySlice {
            inner: de.into_iter()?,
        })
    }

    #[inline(always)]
    fn deserialize_in_place(&mut self, de: Deserializer<'fe>) -> Result<(), Error> {
        self.inner = de.into_iter()?;
        Ok(())
    }
}
