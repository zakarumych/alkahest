use crate::{
    buffer::Buffer,
    deserialize::{Deserialize, DeserializeError, Deserializer},
    formula::{sum_size, BareFormula, Formula},
    serialize::{field_size_hint, write_bytes, write_field, Serialize, Sizes},
};

impl<F> Formula for Option<F>
where
    F: Formula,
{
    const MAX_STACK_SIZE: Option<usize> = sum_size(Some(1), F::MAX_STACK_SIZE);
    const EXACT_SIZE: bool = matches!(F::MAX_STACK_SIZE, Some(0));
    const HEAPLESS: bool = F::HEAPLESS;
}

impl<F> BareFormula for Option<F> where F: Formula {}

impl<F, T> Serialize<Option<F>> for Option<T>
where
    F: Formula,
    T: Serialize<F>,
{
    #[inline(always)]
    fn serialize<B>(self, sizes: &mut Sizes, mut buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        match self {
            None => write_bytes(&[0u8], sizes, buffer),
            Some(value) => {
                write_bytes(&[1u8], sizes, buffer.reborrow())?;
                write_field::<F, T, _>(value, sizes, buffer, true)
            }
        }
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        match self {
            None => {
                let stack = <Option<F>>::MAX_STACK_SIZE?;
                Some(Sizes::with_stack(stack))
            }
            Some(value) => {
                let mut sizes = field_size_hint::<F>(value, true)?;
                sizes.add_stack(1);
                Some(sizes)
            }
        }
    }
}

impl<'ser, F, T> Serialize<Option<F>> for &'ser Option<T>
where
    F: Formula,
    &'ser T: Serialize<F>,
{
    #[inline(always)]
    fn serialize<B>(self, sizes: &mut Sizes, mut buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        match self {
            None => write_bytes(&[0u8], sizes, buffer),
            Some(value) => {
                write_bytes(&[1u8], sizes, buffer.reborrow())?;
                write_field::<F, &T, _>(value, sizes, buffer, true)
            }
        }
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        match *self {
            None => {
                let stack = <Option<F>>::MAX_STACK_SIZE?;
                Some(Sizes::with_stack(stack))
            }
            Some(value) => {
                let mut sizes = field_size_hint::<F>(&value, true)?;
                sizes.add_stack(1);
                Some(sizes)
            }
        }
    }
}

impl<'de, F, T> Deserialize<'de, Option<F>> for Option<T>
where
    F: Formula,
    T: Deserialize<'de, F>,
{
    #[inline(always)]
    fn deserialize(mut de: Deserializer<'de>) -> Result<Self, DeserializeError> {
        let is_some: u8 = de.read_bytes(1)?[0];
        if is_some != 0 {
            Ok(Some(de.read_value::<F, T>(true)?))
        } else {
            Ok(None)
        }
    }

    #[inline(always)]
    fn deserialize_in_place(&mut self, mut de: Deserializer<'de>) -> Result<(), DeserializeError> {
        let is_some: u8 = de.read_bytes(1)?[0];
        if is_some != 0 {
            match self {
                Some(value) => {
                    de.read_in_place::<F, T>(value, true)?;
                }
                None => {
                    *self = Some(de.read_value::<F, T>(true)?);
                }
            }
        } else {
            *self = None;
        }
        Ok(())
    }
}
