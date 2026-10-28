use crate::{
    advanced::size_hint_versioned,
    deserialize::{Deserialize, DeserializeError, Deserializer},
    element::{Element, heap_size_versioned, stack_size_versioned},
    formula::{Formula, SizeBound, SizeType},
    serialize::{Serialize, Serializer, Sizes},
};

pub struct OptionStackSize<E: Element<VERSION>, const SIZE_BYTES: usize, const VERSION: u32 = 0>(E);

impl<E: Element<VERSION>, const SIZE_BYTES: usize, const VERSION: u32> SizeType
    for OptionStackSize<E, SIZE_BYTES, VERSION>
{
    const VALUE: SizeBound = if E::INHABITED {
        stack_size_versioned::<E, SIZE_BYTES, VERSION>()
            .max(SizeBound::Exact(0))
            .add(SizeBound::Exact(1))
    } else {
        SizeBound::Exact(0)
    };
}

pub struct OptionHeapSize<E: Element<VERSION>, const SIZE_BYTES: usize, const VERSION: u32 = 0>(E);

impl<E: Element<VERSION>, const SIZE_BYTES: usize, const VERSION: u32> SizeType
    for OptionHeapSize<E, SIZE_BYTES, VERSION>
{
    const VALUE: SizeBound = if E::INHABITED {
        heap_size_versioned::<E, SIZE_BYTES, VERSION>().max(SizeBound::Exact(0))
    } else {
        // if E is uninhabited, option can only be None, and thus has no heap size
        SizeBound::Exact(0)
    };
}

impl<E, const VERSION: u32> Formula<VERSION> for Option<E>
where
    E: Element<VERSION>,
{
    const SCHEMA: crate::version::Schema =
        crate::version::Schema::new::<Self, VERSION>(crate::version::SchemaKind::Option(E::SCHEMA));
    type StackSize<const SIZE_BYTES: usize> = OptionStackSize<E, SIZE_BYTES, VERSION>;
    type HeapSize<const SIZE_BYTES: usize> = OptionHeapSize<E, SIZE_BYTES, VERSION>;
    const INHABITED: bool = true;
}

impl<E, T, const VERSION: u32> Serialize<Option<E>, VERSION> for Option<T>
where
    E: Element<VERSION>,
    T: Serialize<E::Formula, VERSION>,
{
    #[inline(always)]
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
                simple_try!(serializer.write_bytes(&[1u8]));
                E::serialize(value, &mut serializer)
            }
        }
    }

    #[inline(always)]
    fn size_hint<const SIZE_BYTES: usize>(&self) -> Option<Sizes> {
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
                sizes += simple_some!(size_hint_versioned::<E, T, SIZE_BYTES, VERSION>(value));
                Some(sizes)
            }
        }
    }
}

impl<'de, E, T, const VERSION: u32> Deserialize<'de, Option<E>, VERSION> for Option<T>
where
    E: Element<VERSION>,
    T: Deserialize<'de, E::Formula, VERSION>,
{
    #[inline]
    fn deserialize<D>(mut de: D) -> Result<Self, DeserializeError>
    where
        D: Deserializer<'de>,
    {
        if E::INHABITED {
            let is_some: u8 = simple_try!(de.read_byte());
            if is_some == 0 {
                Ok(None)
            } else {
                let value = simple_try!(E::deserialize(&mut de));
                Ok(Some(value))
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
            let is_some: u8 = simple_try!(de.read_byte());
            if is_some == 0 {
                *self = None;
            } else {
                match self {
                    Some(value) => simple_try!(E::deserialize_in_place(value, &mut de)),
                    None => {
                        let value = simple_try!(E::deserialize(&mut de));
                        *self = Some(value);
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
