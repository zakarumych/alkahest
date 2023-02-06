use crate::{
    deserialize::{Deserialize, Deserializer, Error},
    serialize::{SerializeOwned, Serializer},
};

/// Trait for data formulas.
/// Types that implement this trait are used as markers
/// to guide serialization and deserialization process.
/// Many types that implement `NonRefFormula`
/// implement `SerializeOwned` and/or `Deserialize` traits
/// with `Self` as formula type.
///
/// The typical exceptions are lazily serialized and deserialize types.
/// For example `[T]` can be used as formula for which iterators
/// implement `SerializeOwned` trait.
/// And `SliceIter` and `FromIterator` containers implement `Deserialize` trait.
///
/// Similarly structures that contain `[T]` may be serialized
/// from structures with identical layout but iterator for that field.
///
/// Users may `derive(NonRefFormula)` for their types, structures and enums.
/// Then `derive(SerializeOwned)` and `derive(Deserialize)`
/// will use formula structure to implement serialization and deserialization.
/// Fields of formula structure must be visible in scope of type where
/// `derive(SerializeOwned)` and `derive(Deserialize)` is used.
///
/// Additionally for each field of the serialization/deserialization structure
/// there must be field in formula.
/// And all field of formula structure must be used.
/// Otherwise derive macro will generate compile error.
///
/// Structures can be used to serialize with enum formula.
/// In this case specific variant is used and layout of that variant must
/// match layout of serialization structure.
///
/// Users are also free to implement `NonRefFormula` and other traits manually.
/// In this case they are encouraged to pay attention to `NonRefFormula` documentation.
/// And provide implementations for `SerializeOwned` and `Deserialize` traits
/// with this formula.
///
/// For use-cases outside defining new primitives users are encouraged
/// to use derive macros.
///
/// Implementing traits incorrectly may result in wrong content
/// of serialized data and deserialized values.
/// It can't result in undefined behavior.
pub trait Formula {
    #[doc(hidden)]
    const MAX_SIZE: Option<usize>;

    #[doc(hidden)]
    type NonRef: NonRefFormula + ?Sized;

    #[doc(hidden)]
    fn serialize<T, S>(value: T, serializer: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        T: SerializeOwned<Self::NonRef>,
        S: Serializer;

    #[doc(hidden)]
    fn deserialize<'de, T>(deserializer: Deserializer<'de>) -> Result<T, Error>
    where
        T: Deserialize<'de, Self::NonRef>;

    #[doc(hidden)]
    fn deserialize_in_place<'de, T>(
        place: &mut T,
        deserializer: Deserializer<'de>,
    ) -> Result<(), Error>
    where
        T: Deserialize<'de, Self::NonRef> + ?Sized;
}

/// Function to combine sizes of formulas.
/// Order of arguments is important.
/// First argument is not allowed to be `None` and will cause an error.
/// Second argument may be `None` and will produce `None` as a result.
/// If both arguments are `Some` then result is their sum.
pub const fn combine_sizes(a: Option<usize>, b: Option<usize>) -> Option<usize> {
    let (arr, idx) = match (a, b) {
        (None, _) => ([None], 1), // Error in both runtime and compile time.
        (Some(_), None) => ([None], 0),
        (Some(a), Some(b)) => ([Some(a + b)], 0),
    };
    arr[idx]
}

/// Function for multiplying size of formula by a constant.
/// First argument cannot be `None` and will cause an error.
/// If first argument is `Some` then product of arguments is returned.
pub const fn repeat_size(a: Option<usize>, n: usize) -> Option<usize> {
    let (arr, idx) = match a {
        None => ([None], 1), // Error in both runtime and compile time.
        Some(a) => ([Some(a * n)], 0),
    };
    arr[idx]
}

/// Ad-hoc negative trait.
/// It *should* be implemented for all formulas except [`Ref`]
/// and its aliases, like [`Vec`]
///
/// [`Ref`]: crate::Ref
/// [`Vec`]: alloc::vec::Vec
pub trait NonRefFormula {
    /// Maximum number of bytes serialized values with this formula consume
    /// from "stack" in output buffer.
    ///
    /// Values *may* use less number of bytes.
    /// `Deserialize` implementations must be prepared to handle this.
    ///
    /// Formulas *should* specify as small value as possible.
    /// Providing too large value may result in wasted space in serialized data.
    ///
    /// Unsized formulas like slices should specify `None`.
    /// Same applies for `non_exhaustive` formulas,
    /// as they may be extended in future without breaking
    /// deserialization compatibility.
    #[doc(hidden)]
    const MAX_SIZE: Option<usize>;
}

impl<F> Formula for F
where
    F: NonRefFormula + ?Sized,
{
    type NonRef = Self;

    const MAX_SIZE: Option<usize> = <Self as NonRefFormula>::MAX_SIZE;

    #[doc(hidden)]
    #[cfg_attr(feature = "inline-more", inline(always))]
    fn serialize<T, S>(value: T, serializer: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        T: SerializeOwned<Self::NonRef>,
        S: Serializer,
    {
        T::serialize_owned(value, serializer)
    }

    #[doc(hidden)]
    #[cfg_attr(feature = "inline-more", inline(always))]
    fn deserialize<'de, T>(deserializer: Deserializer<'de>) -> Result<T, Error>
    where
        T: Deserialize<'de, Self::NonRef>,
    {
        T::deserialize(deserializer)
    }

    #[doc(hidden)]
    #[cfg_attr(feature = "inline-more", inline(always))]
    fn deserialize_in_place<'de, T>(
        place: &mut T,
        deserializer: Deserializer<'de>,
    ) -> Result<(), Error>
    where
        T: Deserialize<'de, Self::NonRef> + ?Sized,
    {
        T::deserialize_in_place(place, deserializer)
    }
}
