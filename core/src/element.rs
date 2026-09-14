use crate::{
    deserialize::{Deserialize, DeserializeError, Deserializer},
    formula::{Formula, SizeBound, SizeBytes, SizeType},
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

    #[inline]
    fn serialize<T, S>(value: &T, serializer: &mut S) -> Result<(), S::Error>
    where
        T: Serialize<F> + ?Sized,
        S: Serializer,
    {
        serializer.write_direct::<F, T>(value)
    }

    #[inline]
    fn size_hint<T, const SIZE_BYTES: usize>(value: &T) -> Option<Sizes>
    where
        T: Serialize<F> + ?Sized,
    {
        value.size_hint::<SIZE_BYTES>()
    }

    #[inline]
    fn deserialize<'de, T, D>(deserializer: &mut D) -> Result<T, DeserializeError>
    where
        T: Deserialize<'de, F>,
        D: Deserializer<'de>,
    {
        deserializer.read_value::<F, T>()
    }

    #[inline]
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

pub struct IndirectHeapSize<E: Element + ?Sized, const SIZE_BYTES: usize>(E);

impl<E: Element + ?Sized, const SIZE_BYTES: usize> SizeType for IndirectHeapSize<E, SIZE_BYTES> {
    const VALUE: SizeBound = stack_size::<E, SIZE_BYTES>().add(heap_size::<E, SIZE_BYTES>());
}

impl<E> Element for Indirect<E>
where
    E: Element + ?Sized,
{
    type Formula = E::Formula;

    type StackSize<const SIZE_BYTES: usize> = SizeBytes<SIZE_BYTES>;
    type HeapSize<const SIZE_BYTES: usize> = IndirectHeapSize<E, SIZE_BYTES>;

    const INHABITED: bool = E::INHABITED;

    #[inline]
    fn serialize<T, S>(value: &T, serializer: &mut S) -> Result<(), S::Error>
    where
        T: Serialize<E::Formula> + ?Sized,
        S: Serializer,
    {
        serializer.write_indirect::<E, T>(value)
    }

    #[inline]
    fn size_hint<T, const SIZE_BYTES: usize>(value: &T) -> Option<Sizes>
    where
        T: Serialize<E::Formula> + ?Sized,
    {
        match value.size_hint::<SIZE_BYTES>() {
            None => None,
            Some(sizes) => Some(Sizes {
                stack: SIZE_BYTES,
                heap: sizes.total(),
            }),
        }
    }

    #[inline]
    fn deserialize<'de, T, D>(deserializer: &mut D) -> Result<T, DeserializeError>
    where
        T: Deserialize<'de, E::Formula>,
        D: Deserializer<'de>,
    {
        let address = simple_try!(deserializer.read_usize());
        let mut heap = deserializer.at(address)?;
        E::deserialize(&mut heap)
    }

    #[inline]
    fn deserialize_in_place<'de, T, D>(
        place: &mut T,
        deserializer: &mut D,
    ) -> Result<(), DeserializeError>
    where
        T: Deserialize<'de, E::Formula> + ?Sized,
        D: Deserializer<'de>,
    {
        let address = simple_try!(deserializer.read_usize());
        let mut heap = deserializer.at(address)?;
        E::deserialize_in_place(place, &mut heap)
    }
}

impl<E, T> Serialize<E> for Indirect<T>
where
    E: Element + ?Sized,
    T: Serialize<E::Formula>,
{
    #[inline]
    fn serialize<S>(&self, mut serializer: S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        E::serialize::<T, S>(&self.0, &mut serializer)
    }

    #[inline]
    fn size_hint<const SIZE_BYTES: usize>(&self) -> Option<Sizes> {
        E::size_hint::<T, SIZE_BYTES>(&self.0)
    }
}

impl<'de, E, T> Deserialize<'de, E> for Indirect<T>
where
    E: Element + ?Sized,
    T: Deserialize<'de, E::Formula>,
{
    #[inline]
    fn deserialize<D>(mut deserializer: D) -> Result<Self, DeserializeError>
    where
        D: Deserializer<'de>,
    {
        let value = simple_try!(E::deserialize::<T, D>(&mut deserializer));
        Ok(Indirect(value))
    }

    #[inline]
    fn deserialize_in_place<D>(&mut self, mut deserializer: D) -> Result<(), DeserializeError>
    where
        D: Deserializer<'de>,
    {
        E::deserialize_in_place::<T, D>(&mut self.0, &mut deserializer)
    }
}

#[inline]
pub const fn stack_size<E: Element + ?Sized, const SIZE_BYTES: usize>() -> SizeBound {
    E::StackSize::<SIZE_BYTES>::VALUE
}

#[inline]
pub const fn heap_size<E: Element + ?Sized, const SIZE_BYTES: usize>() -> SizeBound {
    E::HeapSize::<SIZE_BYTES>::VALUE
}

#[inline]
pub const fn inhabited<E: Element + ?Sized>() -> bool {
    E::INHABITED
}
