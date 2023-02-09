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
    const EXACT_SIZE: bool = false;
    const HEAPLESS: bool = F::HEAPLESS;
}

impl<F> NonRefFormula for [F] where F: Formula {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct SerIter<T>(pub T);

impl<F, T, I> Serialize<[F]> for SerIter<I>
where
    F: Formula,
    I: Iterator<Item = T>,
    T: Serialize<F>,
{
    #[inline(always)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut ser = ser.into();
        for elem in self.0 {
            ser.write_value::<F, T>(elem)?;
        }
        ser.finish()
    }

    #[inline(always)]
    fn fast_sizes(&self) -> Option<usize> {
        default_iter_fast_sizes::<F, I>(&self.0)
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

#[inline(always)]
pub fn default_iter_fast_sizes<F, I>(iter: &I) -> Option<usize>
where
    F: Formula,
    I: Iterator,
    I::Item: Serialize<F>,
{
    default_iter_fast_sizes_unchecked::<F, I>(iter)
}

#[inline(always)]
pub fn default_iter_fast_sizes_unchecked<F, I>(iter: &I) -> Option<usize>
where
    F: Formula,
    I: Iterator,
{
    match (F::EXACT_SIZE, F::HEAPLESS, F::MAX_STACK_SIZE) {
        (true, true, Some(max_stack_size)) => {
            let (lower, upper) = iter.size_hint();
            match upper {
                Some(upper) if upper == lower => {
                    // Expect this to be the truth.
                    // If not, serialization will fail or produce incorrect results.
                    Some(lower * max_stack_size)
                }
                _ => None,
            }
        }
        _ => None,
    }
}

#[inline(always)]
pub fn default_iter_fast_sizes_owned<F, T, I>(iter: I) -> Option<usize>
where
    F: Formula,
    I: Iterator<Item = T> + Clone,
    T: Serialize<F>,
{
    match (F::EXACT_SIZE, F::HEAPLESS, F::MAX_STACK_SIZE) {
        (true, true, Some(max_stack_size)) => {
            let (lower, upper) = iter.size_hint();
            match upper {
                Some(upper) if upper == lower => {
                    // Expect this to be the truth.
                    // If not, serialization will fail or produce incorrect results.
                    Some(lower * max_stack_size)
                }
                _ => None,
            }
        }
        _ => {
            let (_, upper) = iter.size_hint();
            match upper {
                Some(upper) if upper <= 4 => {
                    let mut size = 0;
                    for elem in iter {
                        size += <T as Serialize<F>>::fast_sizes(&elem)?;
                    }
                    Some(size)
                }
                _ => None,
            }
        }
    }
}

#[inline(always)]
pub fn default_iter_fast_sizes_by_ref<'a, F, T, I>(iter: I) -> Option<usize>
where
    F: Formula,
    I: Iterator<Item = &'a T>,
    T: Serialize<F> + 'a,
{
    match (F::EXACT_SIZE, F::HEAPLESS, F::MAX_STACK_SIZE) {
        (true, true, Some(max_stack_size)) => {
            let (lower, upper) = iter.size_hint();
            match upper {
                Some(upper) if upper == lower => {
                    // Expect this to be the truth.
                    // If not, serialization will fail or produce incorrect results.
                    Some(lower * max_stack_size)
                }
                _ => None,
            }
        }
        _ => {
            let (_, upper) = iter.size_hint();
            match upper {
                Some(upper) if upper <= 4 => {
                    let mut size = 0;
                    for elem in iter {
                        size += <T as Serialize<F>>::fast_sizes(elem)?;
                    }
                    Some(size)
                }
                _ => None,
            }
        }
    }
}
