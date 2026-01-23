use crate::{
    buffer::Buffer,
    deserialize::{Deserialize, DeserializeError, Deserializer},
    formula::{repeat_size, BareFormulaType, FormulaType},
    iter::{owned_iter_fast_sizes, ref_iter_fast_sizes},
    serialize::{write_array, write_slice, Serialize, SerializeRef, Sizes},
};

impl<F, const N: usize> FormulaType for [F; N]
where
    F: FormulaType,
{
    const MAX_STACK_SIZE: Option<usize> = repeat_size(F::MAX_STACK_SIZE, N);
    const EXACT_SIZE: bool = F::EXACT_SIZE;
    const HEAPLESS: bool = F::HEAPLESS;

    #[cfg(feature = "evolution")]
    fn descriptor(builder: crate::evolution::DescriptorBuilder) {
        builder.sequence::<F>(u32::try_from(N).ok());
    }
}

impl<F, const N: usize> BareFormulaType for [F; N] where F: FormulaType {}

impl<F, T, const N: usize> Serialize<[F; N]> for [T; N]
where
    F: FormulaType,
    T: Serialize<F>,
{
    #[inline(always)]
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        write_array(self.into_iter(), sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        ref_array_fast_sizes::<F, _, _>(self.iter())
    }
}

impl<F, T, const N: usize> SerializeRef<[F; N]> for [T; N]
where
    F: FormulaType,
    for<'ser> &'ser T: Serialize<F>,
{
    #[inline(always)]
    fn serialize<B>(&self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        write_array(self.iter(), sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        owned_array_fast_sizes::<F, _, _>(self.iter())
    }
}

impl<F, T, const N: usize> Serialize<[F]> for [T; N]
where
    F: FormulaType,
    T: Serialize<F>,
{
    #[inline(always)]
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        write_slice(self.into_iter(), sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        ref_iter_fast_sizes::<F, _, _>(self.iter())
    }
}

impl<'ser, F, T, const N: usize> Serialize<[F]> for &'ser [T; N]
where
    F: FormulaType,
    &'ser T: Serialize<F>,
{
    #[inline(always)]
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        write_slice(self.iter(), sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        owned_iter_fast_sizes::<F, _, _>(self.iter())
    }
}

impl<'de, F, T, const N: usize> Deserialize<'de, [F; N]> for [T; N]
where
    F: FormulaType,
    T: Deserialize<'de, F>,
{
    #[inline]
    fn deserialize(mut de: Deserializer<'de>) -> Result<Self, DeserializeError> {
        let mut opts = [(); N].map(|_| None);

        for i in 0..N {
            opts[i] = Some(de.read_value::<F, T>(false)?);
        }

        let value = opts.map(Option::unwrap);
        Ok(value)
    }

    #[inline(always)]
    fn deserialize_in_place(&mut self, mut de: Deserializer<'de>) -> Result<(), DeserializeError> {
        self.iter_mut()
            .try_for_each(|elem| de.read_in_place::<F, T>(elem, false))?;
        Ok(())
    }

    #[cfg(feature = "evolution")]
    #[inline(always)]
    fn deserialize_with_descriptor(
        desc: &crate::evolution::Descriptor,
        formula: u32,
        mut de: Deserializer<'de>,
    ) -> Result<Self, DeserializeError> {
        match *desc.get(formula) {
            crate::evolution::Flavor::Sequence { elem: None, .. } => {
                // Data contains slice.
                let mut opts = [(); N].map(|_| None);

                for i in 0..N {
                    opts[i] = Some(de.read_value::<F, T>(false)?);
                }

                let value = opts.map(Option::unwrap);
                Ok(value)
            }
            crate::evolution::Flavor::Sequence {
                elem: Some(elem), ..
            } => {
                let mut opts = [(); N].map(|_| None);

                for i in 0..N {
                    opts[i] = Some(de.read_value_with_descriptor::<F, T>(desc, elem, false)?);
                }

                let value = opts.map(Option::unwrap);
                Ok(value)
            }
            _ => Err(DeserializeError::Incompatible),
        }
    }
}

/// Returns the size of the serialized data if it can be determined fast.
#[inline(always)]
pub fn owned_array_fast_sizes<F, I, T>(iter: I) -> Option<Sizes>
where
    F: FormulaType + ?Sized,
    I: Iterator<Item = T>,
    T: Serialize<F>,
{
    match (F::HEAPLESS, F::MAX_STACK_SIZE) {
        (true, Some(0)) => Some(Sizes::ZERO),
        _ => owned_iter_fast_sizes::<F, I, T>(iter),
    }
}

/// Returns the size of the serialized data if it can be determined fast.
#[inline(always)]
pub fn ref_array_fast_sizes<'a, F, I, T: 'a>(iter: I) -> Option<Sizes>
where
    F: FormulaType + ?Sized,
    I: Iterator<Item = &'a T>,
    T: Serialize<F>,
{
    match (F::HEAPLESS, F::MAX_STACK_SIZE) {
        (true, Some(0)) => Some(Sizes::ZERO),
        _ => ref_iter_fast_sizes::<F, I, T>(iter),
    }
}
