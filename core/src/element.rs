use crate::{
    deserialize::{Deserialize, DeserializeError, Deserializer},
    formula::{Formula, SizeBound, SizeType},
    serialize::{Serialize, Serializer, Sizes},
};

/// Indirection wrapper for formula elements.
///
/// Instead of implementing formula for this type
/// and then [`Serialize`] and [`Deserialize`] for any type that implements those traits for `F`,
/// it is used as wrapper for element formulas in composite formulas.
///
/// This makes it usage in check, for example it's not possible to wrap formula in `Indirect` twice
/// and get twice indirected formula, because it makes no sense.
pub struct Indirect<E: ?Sized>(pub E);

/// Element of a composite formula.
///
/// It can be either formula or indirected formula.
pub trait Element<const VERSION: u32 = 0>: 'static {
    /// Element's formula type.
    type Formula: Formula<VERSION> + ?Sized;

    /// Formula version and storage mode used by this element.
    const SCHEMA: crate::version::ElementSchema = crate::version::ElementSchema {
        schema: crate::version::unknown_schema::<Self::Formula, VERSION>,
        indirect: false,
    };

    /// Stack size required for serializing this type.
    type StackSize<const SIZE_BYTES: usize>: SizeType + ?Sized;

    /// Heap size required for serializing this type.
    type HeapSize<const SIZE_BYTES: usize>: SizeType + ?Sized;

    const INHABITED: bool;

    /// Serializes value using this element.
    ///
    /// Value must implement `Serialize` for this element's formula.
    fn serialize<T, S>(value: &T, serializer: &mut S) -> Result<(), S::Error>
    where
        T: Serialize<Self::Formula, VERSION> + ?Sized,
        S: Serializer;

    /// Serializes value using this element.
    ///
    /// Value must implement `Serialize` for this element's formula.
    fn serialize_slice<T, S>(slice: &[T], serializer: &mut S) -> Result<(), S::Error>
    where
        T: Serialize<Self::Formula, VERSION>,
        S: Serializer;

    /// Gets size hint for serializing value using this element.
    fn size_hint<T, const SIZE_BYTES: usize>(value: &T) -> Option<Sizes>
    where
        T: Serialize<Self::Formula, VERSION> + ?Sized;

    fn deserialize<'de, T, D>(deserializer: &mut D) -> Result<T, DeserializeError>
    where
        T: Deserialize<'de, Self::Formula, VERSION>,
        D: Deserializer<'de>;

    fn deserialize_slice<'de, T, D>(
        deserializer: &mut D,
        len: usize,
        push: impl FnMut(T),
    ) -> Result<(), DeserializeError>
    where
        T: Deserialize<'de, Self::Formula, VERSION>,
        D: Deserializer<'de>;

    fn deserialize_in_place<'de, T, D>(
        place: &mut T,
        deserializer: &mut D,
    ) -> Result<(), DeserializeError>
    where
        T: Deserialize<'de, Self::Formula, VERSION> + ?Sized,
        D: Deserializer<'de>;
}

impl<F, const VERSION: u32> Element<VERSION> for F
where
    F: Formula<VERSION> + ?Sized,
{
    type Formula = F;

    const SCHEMA: crate::version::ElementSchema = crate::version::ElementSchema {
        schema: crate::version::schema::<F, VERSION>,
        indirect: false,
    };

    type StackSize<const SIZE_BYTES: usize> = <F as Formula<VERSION>>::StackSize<SIZE_BYTES>;
    type HeapSize<const SIZE_BYTES: usize> = <F as Formula<VERSION>>::HeapSize<SIZE_BYTES>;

    const INHABITED: bool = F::INHABITED;

    #[inline(always)]
    fn serialize<T, S>(value: &T, serializer: &mut S) -> Result<(), S::Error>
    where
        T: Serialize<F, VERSION> + ?Sized,
        S: Serializer,
    {
        serializer.write_direct_versioned::<F, T, VERSION>(value)
    }

    #[inline(always)]
    fn serialize_slice<T, S>(slice: &[T], serializer: &mut S) -> Result<(), S::Error>
    where
        T: Serialize<Self::Formula, VERSION>,
        S: Serializer,
    {
        serializer.write_slice_versioned::<F, T, VERSION>(slice)
    }

    #[inline(always)]
    fn size_hint<T, const SIZE_BYTES: usize>(value: &T) -> Option<Sizes>
    where
        T: Serialize<F, VERSION> + ?Sized,
    {
        value.size_hint::<SIZE_BYTES>()
    }

    #[inline(always)]
    fn deserialize<'de, T, D>(deserializer: &mut D) -> Result<T, DeserializeError>
    where
        T: Deserialize<'de, F, VERSION>,
        D: Deserializer<'de>,
    {
        deserializer.read_value_versioned::<F, T, VERSION>()
    }

    #[inline(always)]
    fn deserialize_slice<'de, T, D>(
        deserializer: &mut D,
        len: usize,
        push: impl FnMut(T),
    ) -> Result<(), DeserializeError>
    where
        T: Deserialize<'de, Self::Formula, VERSION>,
        D: Deserializer<'de>,
    {
        deserializer.read_slice_versioned::<F, T, VERSION>(len, push)
    }

    #[inline(always)]
    fn deserialize_in_place<'de, T, D>(
        place: &mut T,
        deserializer: &mut D,
    ) -> Result<(), DeserializeError>
    where
        T: Deserialize<'de, F, VERSION> + ?Sized,
        D: Deserializer<'de>,
    {
        deserializer.read_value_in_place_versioned::<F, T, VERSION>(place)
    }
}

pub struct IndirectStackSize<
    E: Element<VERSION> + ?Sized,
    const SIZE_BYTES: usize,
    const VERSION: u32 = 0,
>(E);

impl<E: Element<VERSION> + ?Sized, const SIZE_BYTES: usize, const VERSION: u32> SizeType
    for IndirectStackSize<E, SIZE_BYTES, VERSION>
{
    const VALUE: SizeBound = if zero_sized::<E, VERSION>() {
        SizeBound::Exact(0)
    } else {
        SizeBound::Exact(SIZE_BYTES)
    };
}

pub struct IndirectHeapSize<
    E: Element<VERSION> + ?Sized,
    const SIZE_BYTES: usize,
    const VERSION: u32 = 0,
>(E);

impl<E: Element<VERSION> + ?Sized, const SIZE_BYTES: usize, const VERSION: u32> SizeType
    for IndirectHeapSize<E, SIZE_BYTES, VERSION>
{
    const VALUE: SizeBound = if zero_sized::<E, VERSION>() {
        SizeBound::Exact(0)
    } else {
        stack_size_versioned::<E, SIZE_BYTES, VERSION>().add(heap_size_versioned::<
            E,
            SIZE_BYTES,
            VERSION,
        >())
    };
}

impl<F, const VERSION: u32> Element<VERSION> for Indirect<F>
where
    F: Formula<VERSION> + ?Sized,
{
    type Formula = F;

    const SCHEMA: crate::version::ElementSchema = crate::version::ElementSchema {
        schema: crate::version::schema::<F, VERSION>,
        indirect: true,
    };

    type StackSize<const SIZE_BYTES: usize> = IndirectStackSize<F, SIZE_BYTES, VERSION>;
    type HeapSize<const SIZE_BYTES: usize> = IndirectHeapSize<F, SIZE_BYTES, VERSION>;

    const INHABITED: bool = F::INHABITED;

    #[inline(always)]
    fn serialize<T, S>(value: &T, serializer: &mut S) -> Result<(), S::Error>
    where
        T: Serialize<F, VERSION> + ?Sized,
        S: Serializer,
    {
        serializer.write_indirect_versioned::<F, T, VERSION>(value)
    }

    #[inline(always)]
    fn serialize_slice<T, S>(slice: &[T], serializer: &mut S) -> Result<(), S::Error>
    where
        T: Serialize<Self::Formula, VERSION>,
        S: Serializer,
    {
        for item in slice {
            simple_try!(serializer.write_indirect_versioned::<F, T, VERSION>(item));
        }

        Ok(())
    }

    #[inline(always)]
    fn size_hint<T, const SIZE_BYTES: usize>(value: &T) -> Option<Sizes>
    where
        T: Serialize<F, VERSION> + ?Sized,
    {
        if const { zero_sized::<F, VERSION>() } {
            return Some(Sizes::ZERO);
        }
        match value.size_hint::<SIZE_BYTES>() {
            None => None,
            Some(sizes) => Some(Sizes {
                stack: SIZE_BYTES,
                heap: sizes.total(),
            }),
        }
    }

    #[inline(always)]
    fn deserialize<'de, T, D>(deserializer: &mut D) -> Result<T, DeserializeError>
    where
        T: Deserialize<'de, F, VERSION>,
        D: Deserializer<'de>,
    {
        if const { zero_sized::<F, VERSION>() } {
            return deserializer.read_value_versioned::<F, T, VERSION>();
        }

        let address = simple_try!(deserializer.read_usize());
        let heap = deserializer.at(address)?;
        <T as Deserialize<F, VERSION>>::deserialize(heap)
    }

    #[inline(always)]
    fn deserialize_slice<'de, T, D>(
        deserializer: &mut D,
        len: usize,
        mut push: impl FnMut(T),
    ) -> Result<(), DeserializeError>
    where
        T: Deserialize<'de, Self::Formula, VERSION>,
        D: Deserializer<'de>,
    {
        for _ in 0..len {
            if const { zero_sized::<F, VERSION>() } {
                let value = simple_try!(deserializer.read_value_versioned::<F, T, VERSION>());
                push(value);
                continue;
            }

            let address = simple_try!(deserializer.read_usize());
            let heap = simple_try!(deserializer.at(address));
            let value = simple_try!(<T as Deserialize<F, VERSION>>::deserialize(heap));
            push(value);
        }

        return Ok(());
    }

    #[inline(always)]
    fn deserialize_in_place<'de, T, D>(
        place: &mut T,
        deserializer: &mut D,
    ) -> Result<(), DeserializeError>
    where
        T: Deserialize<'de, F, VERSION> + ?Sized,
        D: Deserializer<'de>,
    {
        if const { zero_sized::<F, VERSION>() } {
            return deserializer.read_value_in_place_versioned::<F, T, VERSION>(place);
        }

        let address = simple_try!(deserializer.read_usize());
        let heap = deserializer.at(address)?;
        <T as Deserialize<F, VERSION>>::deserialize_in_place(place, heap)
    }
}

impl<F, T, const VERSION: u32> Serialize<F, VERSION> for Indirect<T>
where
    F: Formula<VERSION> + ?Sized,
    T: Serialize<F, VERSION>,
{
    #[inline(always)]
    fn serialize<S>(&self, serializer: S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        <T as Serialize<F, VERSION>>::serialize::<S>(&self.0, serializer)
    }

    #[inline(always)]
    fn size_hint<const SIZE_BYTES: usize>(&self) -> Option<Sizes> {
        <T as Serialize<F, VERSION>>::size_hint::<SIZE_BYTES>(&self.0)
    }
}

impl<'de, F, T, const VERSION: u32> Deserialize<'de, F, VERSION> for Indirect<T>
where
    F: Formula<VERSION> + ?Sized,
    T: Deserialize<'de, F, VERSION>,
{
    #[inline(always)]
    fn deserialize<D>(deserializer: D) -> Result<Self, DeserializeError>
    where
        D: Deserializer<'de>,
    {
        let value = simple_try!(<T as Deserialize<F, VERSION>>::deserialize::<D>(
            deserializer
        ));
        Ok(Indirect(value))
    }

    #[inline(always)]
    fn deserialize_in_place<D>(&mut self, deserializer: D) -> Result<(), DeserializeError>
    where
        D: Deserializer<'de>,
    {
        <T as Deserialize<F, VERSION>>::deserialize_in_place::<D>(&mut self.0, deserializer)
    }
}

#[inline(always)]
pub const fn stack_size<E: Element + ?Sized, const SIZE_BYTES: usize>() -> SizeBound {
    stack_size_versioned::<E, SIZE_BYTES, 0>()
}
/// Version-selecting form of [`stack_size`]. Uses `VERSION` without encoding a version tag.
#[inline(always)]
pub const fn stack_size_versioned<
    E: Element<VERSION> + ?Sized,
    const SIZE_BYTES: usize,
    const VERSION: u32,
>() -> SizeBound {
    E::StackSize::<SIZE_BYTES>::VALUE
}

#[inline(always)]
pub const fn heap_size<E: Element + ?Sized, const SIZE_BYTES: usize>() -> SizeBound {
    heap_size_versioned::<E, SIZE_BYTES, 0>()
}
/// Version-selecting form of [`heap_size`]. Uses `VERSION` without encoding a version tag.
#[inline(always)]
pub const fn heap_size_versioned<
    E: Element<VERSION> + ?Sized,
    const SIZE_BYTES: usize,
    const VERSION: u32,
>() -> SizeBound {
    E::HeapSize::<SIZE_BYTES>::VALUE
}

#[inline(always)]
pub(crate) const fn zero_sized<E: Element<VERSION> + ?Sized, const VERSION: u32>() -> bool {
    stack_size_versioned::<E, 16, VERSION>().is_zero()
        && heap_size_versioned::<E, 16, VERSION>().is_zero()
}

#[inline(always)]
pub const fn inhabited<E: Element + ?Sized>() -> bool {
    inhabited_versioned::<E, 0>()
}
/// Version-selecting form of [`inhabited`]. Uses `VERSION` without encoding a version tag.
#[inline(always)]
pub const fn inhabited_versioned<E: Element<VERSION> + ?Sized, const VERSION: u32>() -> bool {
    E::INHABITED
}
