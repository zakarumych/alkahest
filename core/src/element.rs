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
pub struct Indirect<F: ?Sized>(F);

/// Element of a composite formula.
///
/// It can be either formula or indirected formula.
pub trait Element: 'static {
    /// Element's formula type.
    type Formula: Formula + ?Sized;

    /// Stack size required for serializing this type.
    type StackSize<const SIZE_BYTES: u8>: SizeType + ?Sized;

    /// Heap size required for serializing this type.
    type HeapSize<const SIZE_BYTES: u8>: SizeType + ?Sized;

    const INHABITED: bool;

    /// Serializes value using this element.
    ///
    /// Value must implement `Serialize` for this element's formula.
    fn serialize<T, S>(value: &T, serializer: &mut S) -> Result<(), S::Error>
    where
        T: Serialize<Self::Formula> + ?Sized,
        S: Serializer;

    /// Gets size hint for serializing value using this element.
    fn size_hint<T, const SIZE_BYTES: u8>(value: &T) -> Option<Sizes>
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

    type StackSize<const SIZE_BYTES: u8> = <F as Formula>::StackSize<SIZE_BYTES>;
    type HeapSize<const SIZE_BYTES: u8> = <F as Formula>::HeapSize<SIZE_BYTES>;

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
    fn size_hint<T, const SIZE_BYTES: u8>(value: &T) -> Option<Sizes>
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
        deserializer.read_direct::<F, T>()
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
        deserializer.read_direct_in_place::<F, T>(place)
    }
}

pub struct IndirectHeapSize<F: Formula + ?Sized, const SIZE_BYTES: u8>(F);

impl<F: Formula + ?Sized, const SIZE_BYTES: u8> SizeType for IndirectHeapSize<F, SIZE_BYTES> {
    const VALUE: SizeBound = stack_size::<F, SIZE_BYTES>().add(heap_size::<F, SIZE_BYTES>());
}

impl<F> Element for Indirect<F>
where
    F: Formula + ?Sized,
{
    type Formula = F;

    type StackSize<const SIZE_BYTES: u8> = SizeBytes<SIZE_BYTES>;
    type HeapSize<const SIZE_BYTES: u8> = IndirectHeapSize<F, SIZE_BYTES>;

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
    fn size_hint<T, const SIZE_BYTES: u8>(value: &T) -> Option<Sizes>
    where
        T: Serialize<F> + ?Sized,
    {
        let heap = value.size_hint::<SIZE_BYTES>()?.total();
        Some(Sizes {
            stack: usize::from(SIZE_BYTES),
            heap,
        })
    }

    #[inline(always)]
    fn deserialize<'de, T, D>(deserializer: &mut D) -> Result<T, DeserializeError>
    where
        T: Deserialize<'de, F>,
        D: Deserializer<'de>,
    {
        deserializer.read_indirect::<F, T>()
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
        deserializer.read_indirect_in_place::<F, T>(place)
    }
}

pub const fn stack_size<E: Element + ?Sized, const SIZE_BYTES: u8>() -> SizeBound {
    E::StackSize::<SIZE_BYTES>::VALUE
}

pub const fn heap_size<E: Element + ?Sized, const SIZE_BYTES: u8>() -> SizeBound {
    E::HeapSize::<SIZE_BYTES>::VALUE
}

pub const fn inhabited<E: Element + ?Sized>() -> bool {
    E::INHABITED
}
