/// Trait for data formulas.
/// Types that implement this trait are used as markers
/// to guide serialization and deserialization process.
/// Many types that implement `Formula`
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
/// Users may `derive(Formula)` for their types, structures and enums.
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
/// Users are also free to implement `Formula` and other traits manually.
/// In this case they are encouraged to pay attention to `Formula` documentation.
/// And provide implementations for `Serialize` and `Deserialize` traits
/// with this formula.
///
/// For use-cases outside defining new primitives users are encouraged
/// to use derive macros.
///
/// Implementing traits incorrectly may result in wrong content
/// of serialized data and deserialized values.
/// It can't result in undefined behavior.
pub trait UnsizedFormula {}

/// Trait similar to `Formula` implemented by types
/// for which size is known in advance.
pub trait Formula: UnsizedFormula {
    /// Size in bytes of serialized value with this formula.
    const SIZE: usize;
}

pub trait FormulaAlias {
    type Alias;
}

impl<S, A> UnsizedFormula for S
where
    A: UnsizedFormula,
    S: FormulaAlias<Alias = A>,
{
}

impl<S, A> Formula for S
where
    A: Formula,
    S: FormulaAlias<Alias = A>,
{
    const SIZE: usize = A::SIZE;
}
