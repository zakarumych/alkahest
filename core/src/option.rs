use crate::{
    deserialize::{Deserialize, DeserializeError, Deserializer},
    element::{Element, heap_size, stack_size},
    formula::{Formula, SizeBound, SizeType},
    serialize::{Serialize, Serializer, Sizes},
};

pub struct OptionStackSize<E: Element, const SIZE_BYTES: u8>(E);

impl<E: Element, const SIZE_BYTES: u8> SizeType for OptionStackSize<E, SIZE_BYTES> {
    const VALUE: SizeBound = if E::INHABITED {
        stack_size::<E, SIZE_BYTES>().add(SizeBound::Exact(1))
    } else {
        SizeBound::Exact(0)
    };
}

pub struct OptionHeapSize<E: Element, const SIZE_BYTES: u8>(E);

impl<E: Element, const SIZE_BYTES: u8> SizeType for OptionHeapSize<E, SIZE_BYTES> {
    const VALUE: SizeBound = if E::INHABITED {
        heap_size::<E, SIZE_BYTES>()
    } else {
        // if E is uninhabited, option can only be None, and thus has no heap size
        SizeBound::Exact(0)
    };
}

impl<E> Formula for Option<E>
where
    E: Element,
{
    type StackSize<const SIZE_BYTES: u8> = OptionStackSize<E, SIZE_BYTES>;
    type HeapSize<const SIZE_BYTES: u8> = OptionHeapSize<E, SIZE_BYTES>;
    const INHABITED: bool = true;
}

impl<E, T> Serialize<Option<E>> for Option<T>
where
    E: Element,
    T: Serialize<E::Formula>,
{
    #[inline]
    fn serialize<S>(&self, mut serializer: S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        match self {
            None => {
                if E::INHABITED {
                    serializer.write_bytes(&[0u8])
                } else {
                    // Do not serialize anything for None of uninhabited option type
                    Ok(())
                }
            }
            Some(value) => {
                debug_assert!(
                    E::INHABITED,
                    "Cannot serialize Some(_) for uninhabited option type"
                );
                serializer.write_bytes(&[1u8])?;
                E::serialize(value, &mut serializer)
            }
        }
    }

    #[inline]
    fn size_hint<const SIZE_BYTES: u8>(&self) -> Option<Sizes> {
        match self {
            None => Some(if E::INHABITED {
                Sizes::with_stack(1)
            } else {
                Sizes::ZERO
            }),
            Some(value) => {
                debug_assert!(
                    E::INHABITED,
                    "Cannot serialize Some(_) for uninhabited option type"
                );
                let mut sizes = Sizes::with_stack(1);
                sizes += E::size_hint::<T, SIZE_BYTES>(value)?;
                Some(sizes)
            }
        }
    }
}

impl<'de, E, T> Deserialize<'de, Option<E>> for Option<T>
where
    E: Element,
    T: Deserialize<'de, E::Formula>,
{
    #[inline]
    fn deserialize<D>(mut de: D) -> Result<Self, DeserializeError>
    where
        D: Deserializer<'de>,
    {
        if E::INHABITED {
            let is_some: u8 = de.read_byte()?;
            if is_some == 0 {
                Ok(None)
            } else {
                E::deserialize(&mut de).map(Some)
            }
        } else {
            // For uninhabited option type, we can only have None
            Ok(None)
        }
    }

    #[inline]
    fn deserialize_in_place<D>(&mut self, mut de: D) -> Result<(), DeserializeError>
    where
        D: Deserializer<'de>,
    {
        if E::INHABITED {
            let is_some: u8 = de.read_byte()?;
            if is_some == 0 {
                *self = None;
            } else {
                match self {
                    Some(value) => E::deserialize_in_place(value, &mut de)?,
                    None => {
                        *self = Some(E::deserialize(&mut de)?);
                    }
                }
            }
        } else {
            // For uninhabited option type, we can only have None
            *self = None;
        }
        Ok(())
    }
}
