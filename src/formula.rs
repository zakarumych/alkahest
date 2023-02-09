/// Trait for data formulas.
/// Types that implement this trait are used as markers
/// to guide serialization and deserialization process.
/// Many types that implement `NonRefFormula`
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
/// Users may `derive(NonRefFormula)` for their types, structures and enums.
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
/// Users are also free to implement `NonRefFormula` and other traits manually.
/// In this case they are encouraged to pay attention to `NonRefFormula` documentation.
/// And provide implementations for `Serialize` and `Deserialize` traits
/// with this formula.
///
/// For use-cases outside defining new primitives users are encouraged
/// to use derive macros.
///
/// Implementing traits incorrectly may result in wrong content
/// of serialized data and deserialized values.
/// It can't result in undefined behavior.
pub trait Formula {
    /// Maximum size of stack this formula occupies.
    const MAX_STACK_SIZE: Option<usize>;

    /// Signals that `MAX_STACK_SIZE` is accurate.
    const EXACT_SIZE: bool;

    /// Signals that heap is not used for serialzation.
    const HEAPLESS: bool;
}

/// Ad-hoc negative trait.
/// It *should* be implemented for all formulas except [`Ref`]
/// and its aliases, like [`Vec`]
///
/// [`Ref`]: crate::Ref
/// [`Vec`]: alloc::vec::Vec
pub trait NonRefFormula: Formula {}

/// Function to combine sizes of formulas.
/// Order of arguments is important.
/// First argument is not allowed to be `None` and will cause an error.
/// Second argument may be `None` and will produce `None` as a result.
/// If both arguments are `Some` then result is their sum.
pub const fn sum_size(a: Option<usize>, b: Option<usize>) -> Option<usize> {
    let (arr, idx) = match (a, b) {
        (None, _) => ([None], 1), // Error in both runtime and compile time.
        (Some(_), None) => ([None], 0),
        (Some(a), Some(b)) => ([Some(a + b)], 0),
    };
    arr[idx]
}

/// Function to combine sizes of formulas.
/// Order of arguments is not important.
/// If any argument is `None` then result is `None`.
/// If both arguments are `Some` then result is maximum of the two.
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
pub const fn repeat_size(a: Option<usize>, n: usize) -> Option<usize> {
    let (arr, idx) = match a {
        None => ([None], 1), // Error in both runtime and compile time.
        Some(a) => ([Some(a * n)], 0),
    };
    arr[idx]
}
