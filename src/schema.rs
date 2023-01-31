/// Trait for data schemas.
/// Types that implement this trait are used as markers
/// to guide serialization and deserialization process.
/// Many types that implement `Schema`
/// implement `Serialize` and/or `Deserialize` traits
/// with `Self` as schema type.
///
/// The typical exceptions are lazily serialized and deserialize types.
/// For example `[T]` can be used as schema for which iterators
/// implement `Serialize` trait.
/// And `SliceIter` and `FromIterator` containers implement `Deserialize` trait.
///
/// Similarly structures that contain `[T]` may be serialized
/// from structures with identical layout but iterator for that field.
///
/// Users may `derive(Schema)` for their types, structures and enums.
/// Then `derive(Serialize)` and `derive(Deserialize)`
/// will use schema structure to implement serialization and deserialization.
/// Fields of schema structure must be visible in scope of type where
/// `derive(Serialize)` and `derive(Deserialize)` is used.
///
/// Additionally for each field of the serialization/deserialization structure
/// there must be field in schema.
/// And all field of schema structure must be used.
/// Otherwise derive macro will generate compile error.
///
/// Structures can be used to serialize with enum schema.
/// In this case specific variant is used and layout of that variant must
/// match layout of serialization structure.
///
/// Users are also free to implement `Schema` and other traits manually.
/// In this case they are encouraged to pay attention to `Schema` documentation.
/// And provide implementations for `Serialize` and `Deserialize` traits
/// with this schema.
///
/// For use-cases outside defining new primitives users are encouraged
/// to use derive macros.
///
/// Implementing traits incorrectly may result in wrong content
/// of serialized data and deserialized values.
/// It can't result in undefined behavior.
pub trait Schema {}

/// Trait similar to `Schema` implemented by types
/// for which size is known in advance.
pub trait SizedSchema: Schema {
    /// Size in bytes that needs to serialize value with this schema.
    const SIZE: usize;
}
