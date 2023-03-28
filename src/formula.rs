use core::mem::size_of;

use crate::size::FixedUsize;

/// Trait for data formulas.
/// Types that implement this trait are used as markers
/// to guide serialization and deserialization process.
/// Many types that implement `BareFormula`
/// implement `Serialize` and/or `Deserialize` traits
/// with `Self` as formula type.
///
/// The typical exceptions are lazily serialized and deserialize types.
/// For example `[T]` can be used as formula for which iterators
/// implement `Serialize` trait.
/// And `SliceIter` and `FromIterator` containers implement `Deserialize` trait.
///
/// Similarly structures that contain `[T]` may be serialized
/// from structures with identical layout but iterator for that field.
///
/// Users may `derive(BareFormula)` for their types, structures and enums.
/// Then `derive(Serialize)` and `derive(Deserialize)`
/// will use formula structure to implement serialization and deserialization.
/// Fields of formula structure must be visible in scope of type where
/// `derive(Serialize)` and `derive(Deserialize)` is used.
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
/// Users are also free to implement `BareFormula` and other traits manually.
/// In this case they are encouraged to pay attention to `BareFormula` documentation.
/// And provide implementations for `Serialize` and `Deserialize` traits
/// with this formula.
///
/// For use-cases outside defining new primitives users are encouraged
/// to use derive macros.
///
/// Implementing traits incorrectly may result in wrong content
/// of serialized data and deserialized values.
/// It can't result in undefined behavior.
///
/// # Examples
///
/// ```
/// # use alkahest::*;
///
/// struct MyFormula;
///
/// impl Formula for MyFormula {
///     const MAX_STACK_SIZE: Option<usize> = Some(0);
///     const EXACT_SIZE: bool = true;
///     const HEAPLESS: bool = true;
/// }
/// ```
#[cfg_attr(
    feature = "derive",
    doc = r#"

When "derive" feature is enabled, `derive(Formula)` is also available.

```
# use alkahest::*;

/// Formula for serializing unit structures.
#[derive(Formula)]
struct UnitFormula;


# #[cfg(feature = "alloc")]
/// Formula for serializing tuple structures with fields
/// that are serializable with `u8` and `String` formulas.
#[derive(Formula)]
struct TupleFormula(u8, String);

# #[cfg(feature = "alloc")]
/// Formula for serializing structures with fields
/// that are serializable with `TupleFormula` and `Vec<usize>` formulas.
#[derive(Formula)]
struct StructFormula {
    a: TupleFormula,
    b: Vec<u32>,
}


# #[cfg(feature = "alloc")]
/// Formula for serializing enums.
#[derive(Formula)]
enum EnumFormula {
    A,
    B(StructFormula),
    C { y: String },
}
```

Names of the formula variants and fields are important for `Serialize` and `Deserialize` derive macros.
"#
)]
pub trait Formula {
    /// Maximum size of stack this formula occupies.
    const MAX_STACK_SIZE: Option<usize>;

    /// Signals that `MAX_STACK_SIZE` is accurate.
    const EXACT_SIZE: bool;

    /// Signals that heap is not used for serialzation.
    const HEAPLESS: bool;
}

/// Ad-hoc negative trait.
/// It should be implemented for most formulas.
/// Except for formulas with generic implementation of `Serialize` and `Deserialize` traits
/// via another `Formula`.
///
/// [`Ref`], [`Vec`], [`String`], [`As`] are examples of such formulas.
///
/// [`Ref`]: crate::Ref
/// [`Vec`]: alloc::vec::Vec
/// [`String`]: alloc::string::String
/// [`As`]: crate::As
pub trait BareFormula: Formula {}

#[inline(always)]
#[track_caller]
pub(crate) const fn unwrap_size(a: Option<usize>) -> usize {
    let (arr, idx) = match a {
        None => ([0], 1), // DeserializeError in both runtime and compile time.
        Some(a) => ([a], 0),
    };
    arr[idx]
}

/// Function to combine sizes of formulas.
/// If any of two is `None` then result is `None`.
#[inline(always)]
#[doc(hidden)]
pub const fn sum_size(a: Option<usize>, b: Option<usize>) -> Option<usize> {
    match (a, b) {
        (None, _) | (_, None) => None,
        (Some(a), Some(b)) => Some(a + b),
    }
}

/// Function to combine sizes of formulas.
/// Order of arguments is not important.
/// If any argument is `None` then result is `None`.
/// If both arguments are `Some` then result is maximum of the two.
#[inline(always)]
#[doc(hidden)]
pub const fn max_size(a: Option<usize>, b: Option<usize>) -> Option<usize> {
    match (a, b) {
        (None, _) => None,
        (Some(_), None) => None,
        (Some(a), Some(b)) if a > b => Some(a),
        (Some(_), Some(b)) => Some(b),
    }
}

/// Function for multiplying size of formula by a constant.
/// First argument cannot be `None` and will cause an error.
/// If first argument is `Some` then product of arguments is returned.
#[inline(always)]
pub(crate) const fn repeat_size(a: Option<usize>, n: usize) -> Option<usize> {
    match a {
        None => None,
        Some(a) => Some(a * n),
    }
}

/// Returns size of formula reference.
#[inline(always)]
pub const fn reference_size<F>() -> usize
where
    F: Formula + ?Sized,
{
    match (F::MAX_STACK_SIZE, F::EXACT_SIZE) {
        (Some(0), _) => 0,
        (Some(_), true) => size_of::<FixedUsize>(),
        _ => size_of::<[FixedUsize; 2]>(),
    }
}
