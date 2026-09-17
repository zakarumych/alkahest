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
pub trait Element: 'static {
    /// Element's formula type.
    type Formula: Formula + ?Sized;

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
        T: Serialize<Self::Formula> + ?Sized,
        S: Serializer;

    /// Gets size hint for serializing value using this element.
    fn size_hint<T, const SIZE_BYTES: usize>(value: &T) -> Option<Sizes>
    where
        T: Serialize<Self::Formula> + ?Sized;

    fn deserialize<'de, T, D>(deserializer: &mut D) -> Result<T, DeserializeError>
    where
        T: Deserialize<'de, Self::Formula>,
        D: Deserializer<'de>;

    fn deserialize_in_place<'de, T, D>(
        place: &mut T,
        deserializer: &mut D,
    ) -> Result<(), DeserializeError>
    where
        T: Deserialize<'de, Self::Formula> + ?Sized,
        D: Deserializer<'de>;
}

impl<F> Element for F
where
    F: Formula + ?Sized,
{
    type Formula = F;

    type StackSize<const SIZE_BYTES: usize> = <F as Formula>::StackSize<SIZE_BYTES>;
    type HeapSize<const SIZE_BYTES: usize> = <F as Formula>::HeapSize<SIZE_BYTES>;

    const INHABITED: bool = F::INHABITED;

    #[inline(always)]
    fn serialize<T, S>(value: &T, serializer: &mut S) -> Result<(), S::Error>
    where
        T: Serialize<F> + ?Sized,
        S: Serializer,
    {
        serializer.write_direct::<F, T>(value)
    }

    #[inline(always)]
    fn size_hint<T, const SIZE_BYTES: usize>(value: &T) -> Option<Sizes>
    where
        T: Serialize<F> + ?Sized,
    {
        value.size_hint::<SIZE_BYTES>()
    }

    #[inline(always)]
    fn deserialize<'de, T, D>(deserializer: &mut D) -> Result<T, DeserializeError>
    where
        T: Deserialize<'de, F>,
        D: Deserializer<'de>,
    {
        deserializer.read_value::<F, T>()
    }

    #[inline(always)]
    fn deserialize_in_place<'de, T, D>(
        place: &mut T,
        deserializer: &mut D,
    ) -> Result<(), DeserializeError>
    where
        T: Deserialize<'de, F> + ?Sized,
        D: Deserializer<'de>,
    {
        deserializer.read_value_in_place::<F, T>(place)
    }
}

pub struct IndirectStackSize<E: Element + ?Sized, const SIZE_BYTES: usize>(E);

impl<E: Element + ?Sized, const SIZE_BYTES: usize> SizeType for IndirectStackSize<E, SIZE_BYTES> {
    const VALUE: SizeBound = if zero_sized::<E>() {
        SizeBound::Exact(0)
    } else {
        SizeBound::Exact(SIZE_BYTES)
    };
}

pub struct IndirectHeapSize<E: Element + ?Sized, const SIZE_BYTES: usize>(E);

impl<E: Element + ?Sized, const SIZE_BYTES: usize> SizeType for IndirectHeapSize<E, SIZE_BYTES> {
    const VALUE: SizeBound = if zero_sized::<E>() {
        SizeBound::Exact(0)
    } else {
        stack_size::<E, SIZE_BYTES>().add(heap_size::<E, SIZE_BYTES>())
    };
}

impl<F> Element for Indirect<F>
where
    F: Formula + ?Sized,
{
    type Formula = F;

    type StackSize<const SIZE_BYTES: usize> = IndirectStackSize<F, SIZE_BYTES>;
    type HeapSize<const SIZE_BYTES: usize> = IndirectHeapSize<F, SIZE_BYTES>;

    const INHABITED: bool = F::INHABITED;

    #[inline(always)]
    fn serialize<T, S>(value: &T, serializer: &mut S) -> Result<(), S::Error>
    where
        T: Serialize<F> + ?Sized,
        S: Serializer,
    {
        serializer.write_indirect::<F, T>(value)
    }

    #[inline(always)]
    fn size_hint<T, const SIZE_BYTES: usize>(value: &T) -> Option<Sizes>
    where
        T: Serialize<F> + ?Sized,
    {
        if const { zero_sized::<F>() } {
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
        T: Deserialize<'de, F>,
        D: Deserializer<'de>,
    {
        if const { zero_sized::<F>() } {
            return deserializer.read_value::<F, T>();
        }

        let address = simple_try!(deserializer.read_usize());
        let heap = deserializer.at(address)?;
        <T as Deserialize<F>>::deserialize(heap)
    }

    #[inline(always)]
    fn deserialize_in_place<'de, T, D>(
        place: &mut T,
        deserializer: &mut D,
    ) -> Result<(), DeserializeError>
    where
        T: Deserialize<'de, F> + ?Sized,
        D: Deserializer<'de>,
    {
        if const { zero_sized::<F>() } {
            return deserializer.read_value_in_place::<F, T>(place);
        }

        let address = simple_try!(deserializer.read_usize());
        let heap = deserializer.at(address)?;
        <T as Deserialize<F>>::deserialize_in_place(place, heap)
    }
}

impl<F, T> Serialize<F> for Indirect<T>
where
    F: Formula + ?Sized,
    T: Serialize<F>,
{
    #[inline(always)]
    fn serialize<S>(&self, serializer: S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        <T as Serialize<F>>::serialize::<S>(&self.0, serializer)
    }

    #[inline(always)]
    fn size_hint<const SIZE_BYTES: usize>(&self) -> Option<Sizes> {
        <T as Serialize<F>>::size_hint::<SIZE_BYTES>(&self.0)
    }
}

impl<'de, F, T> Deserialize<'de, F> for Indirect<T>
where
    F: Formula + ?Sized,
    T: Deserialize<'de, F>,
{
    #[inline(always)]
    fn deserialize<D>(deserializer: D) -> Result<Self, DeserializeError>
    where
        D: Deserializer<'de>,
    {
        let value = simple_try!(<T as Deserialize<F>>::deserialize::<D>(deserializer));
        Ok(Indirect(value))
    }

    #[inline(always)]
    fn deserialize_in_place<D>(&mut self, deserializer: D) -> Result<(), DeserializeError>
    where
        D: Deserializer<'de>,
    {
        <T as Deserialize<F>>::deserialize_in_place::<D>(&mut self.0, deserializer)
    }
}

#[inline(always)]
pub const fn stack_size<E: Element + ?Sized, const SIZE_BYTES: usize>() -> SizeBound {
    E::StackSize::<SIZE_BYTES>::VALUE
}

#[inline(always)]
pub const fn heap_size<E: Element + ?Sized, const SIZE_BYTES: usize>() -> SizeBound {
    E::HeapSize::<SIZE_BYTES>::VALUE
}

#[inline(always)]
pub(crate) const fn zero_sized<E: Element + ?Sized>() -> bool {
    stack_size::<E, 16>().is_zero() && heap_size::<E, 16>().is_zero()
}

#[inline(always)]
pub const fn inhabited<E: Element + ?Sized>() -> bool {
    E::INHABITED
}
