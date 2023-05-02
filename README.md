# Alkahest - Fantastic serialization library.

[![crates](https://img.shields.io/crates/v/alkahest.svg?style=for-the-badge&label=alkahest)](https://crates.io/crates/alkahest)
[![docs](https://img.shields.io/badge/docs.rs-alkahest-66c2a5?style=for-the-badge&labelColor=555555&logoColor=white)](https://docs.rs/alkahest)
[![actions](https://img.shields.io/github/actions/workflow/status/zakarumych/alkahest/badge.yml?branch=main&style=for-the-badge)](https://github.com/zakarumych/alkahest/actions/workflows/badge.yml)
[![MIT/Apache](https://img.shields.io/badge/license-MIT%2FApache-blue.svg?style=for-the-badge)](./COPYING)
![loc](https://img.shields.io/tokei/lines/github/zakarumych/alkahest?style=for-the-badge)

*Alkahest* is blazing-fast, zero-deps, zero-overhead, zero-unsafe, schema-based
serialization library.
It is suitable for broad range of use-cases, but tailored for
custom high-performance network protocols.

### Benchmarks

This benchmark that mimics some game networking protocol.

|                 | `alkahest`               | `bincode`                       | `rkyv`                          | `speedy`                         |
|:----------------|:-------------------------|:--------------------------------|:--------------------------------|:-------------------------------- |
| **`serialize`** | `10.69 us` (✅ **1.00x**) | `11.08 us` (✅ **1.04x slower**) | `12.43 us` (❌ *1.16x slower*)   | `11.13 us` (✅ **1.04x slower**)  |
| **`read`**      | `1.19 us` (✅ **1.00x**)  | `9.19 us` (❌ *7.74x slower*)    | `2.10 us` (❌ *1.77x slower*)    | `1.54 us` (❌ *1.30x slower*)     |

---
Made with [criterion-table](https://github.com/nu11ptr/criterion-table)


See also [benchmark results](./BENCHMARKS.md) from <https://github.com/djkoloski/rust_serialization_benchmark> (in draft until 0.2 release).

## Features

* **Schema-based serialization**.
  Alkahest uses data schemas called `Formula`s to serialize and deserialize data.
  Thus controlling data layout independently from data types that are serialized
  or deserialized.

* **Support wide variety of formulas**.
  Integers, floats, booleans, tuples, arrays, slices, strings and
  user-defined formulas with custom data layout using `derive` macro
  that works for structs and enums of any complexity and supports generics.

* **Zero-overhead serialization of sequences**.
  Alkahest support serializing iterators directly into slice formulas.
  No more allocation of a `Vec` to serialize and drop immediately.

* **Lazy deserialization**.
  Alkahest provides `Lazy<F>` type to deserialize any formula `F` lazily.
  `Lazy` can be used later to perform actual deserialization.\
  `Lazy<[F]>` can also produce iterator that deserializes elements on demand.\
  Laziness is controlled on type level and can be applied to any element
  of a larger formula.

* **Infallible serialization**.
  Given large enough or growing buffer any value that implements `Serialize`
  can be serialized without error.
  No more unnecessary unwraps or puzzles "what to do if serialization fails?".
  The only error condition for serialization is "data doesn't fit".

### Planned features

* Serializable formula descriptors
* Compatibility rules
* External tool for code-generation for formula descriptors for C and Rust.

## How it works. In more details

*Alkahest* separates data schema definition (aka `Formula`) from
serialization and deserialization code.
Doing so, this library provides better guarantees for cases
when serializable data type and deserializable data type
are different.
It also supports serializing from iterators instead of collections
and deserialization into lazy wrappers that defers costly process
and may omit it entirely if value is never accessed.
User controls laziness on type level by choosing appropriate `Deserialize` impls.
For instance deserializing into `Vec<T>` is eager because `Vec<T>` is constructed
with all `T` instances and memory allocated for them.
While `alkahest::SliceIter` implements `Iterator` and deserializes
elements in `Iterator::next` and other methods. And provides constant-time
random access to any element.

Flexibility comes at cost of using only byte slices for
serialization and deserialization.
And larger footprint of serialized data than some other binary formats.

Question about support of dense data packing is open.
It may be desireable to control on type level.

### Errors and panics

The API is designed with following principles:
Any value can be serialized successfully given large enough buffer.
Data can't cause panic, incorrect implementation of a trait can.

There is *zero* unsafe code in the library on any code it generates.
No UB is possible given that `std` is not unsound.

### Forward and backward compatibility

No data schemas stays the same.
New fields and variants are added,
others are deprecated and removed.

There's set of rules that ensures forward compatibility between formulas.
And another set or rules for backward compatibility.

Verification of compatibility is not implemented yet.

### Forward compatibility

Forward compatibility is an ability to deserialize data
that was serialized with newer formulas.

TODO: List all rules

### Backward compatibility

Backward compatibility is an ability to deserialize data
that was serialized with older formulas.

TODO: List all rules

## Formula, Serialize and Deserialize traits.

The crate works using three fundamental traits.
`Formula`, `Serialize` and `Deserialize`.
There's also supporting trait - `BareFormula`.

*Alkahest* provides proc-macro `alkahest` for deriving `Formula`, `Serialize` and `Deserialize`.

### Formula

`Formula` trait is used to allow types to serve as data schemas.
Any value serialized with given formula should be deserializable with the same
formula. Sharing only `Formula` type allows modules and crates
easily communicate.
`Formula` dictates binary data layout and it *must* be platform-independent.

Potentially `Formula` types can be generated from separate files,
opening possibility for cross-language communication.

`Formula` is implemented for a number of types out-of-the-box.
Primitive types like `bool`, integers and floating point types all implement `Formula`.
This excludes `isize` and `usize`.
In their place there's `FixedUsize` and `FixedIsize` types provided,
whose size is controlled by a feature-flag.
*!Caveat!*:
  Sizes and addresses are serialized as `FixedUsize`.
  Truncating `usize` value if it was too large.
  This may result in broken data generated and panic in debug.
  Increase size of the `FixedUsize` if you encounter this.
It is also implemented for tuples, array and slice, `Option` and `Vec` (the later requires `"alloc"` feature).

The easiest way to define a new formula is to derive `Formula` trait for a struct or an enum.
Generics are supported, but may require complex bounds specified in attributes for
`Serialize` and `Deserialize` derive macros.
The only constrain is that all fields must implement `Formula`.
### Serialize

`Serialize<Formula>` trait is used to implement serialization
according to a specific formula.
Serialization writes to mutable bytes slice and *should not*
perform dynamic allocations.
Binary result of any type serialized with a formula must follow it.
At the end, if a stream of primitives serialized is the same,
binary result should be the same.
Types may be serializable with different formulas producing
different binary result.

`Serialize` is implemented for many types.
Most notably there's implementation `T: Serialize<T>`
and `&T: Serialize<T>` for all primitives `T` (except `usize` and `isize`).
Another important implementation is
`Serialize<F> for I where I: IntoIterator, I::Item: Serialize<F>`,
allowing serializing into slice directly from both iterators and collections.
Serialization with formula `Ref<F>` uses serialization with formula `F`
and then stores relative address and size. No dynamic allocations is required.

Deriving `Serialize` for a type will generate `Serialize` implementation,
formula is specified in attribute `#[alkahest(FormulaRef)]` or
`#[alkahest(serialize(FormulaRef))]`. `FormulaRef` is typically a type.
When generics are used it also contains generic parameters and bounds.
If formula is not specified - `Self` is assumed.
`Formula` should be derived for the type as well.
It is in-advised to derive `Serialize` for formulas with
manual `Formula` implementation,
`Serialize` derive macro generates code that uses non-public items
generated by `Formula` derive macro.
So either both *should have* manual implementation or both derived.

For structures `Serialize` derive macro requires that all fields
are present on both `Serialize` and `Formula` structure and has the same
order (trivially if this is the same structure).

For enums `Serialize` derive macro checks that for each variant there
exists variant on `Formula` enum.
Variants content is compared similar to structs.
Serialization inserts variant ID and serializes variant as struct.
The size of variants may vary. Padding is inserted by outer value serialization
if necessary.

`Serialize` can be derived for structure where `Formula` is an enum.
In this case variant should be specified using
`#[alkahest(@variant_ident)]` or `#[alkahest(serialize(@variant_ident))]`
and then `Serialize` derive macro will produce serialization code that works
as if this variant was a struct `Formula`,
except that variant's ID will be serialized before fields.

`Serialize` can be derived for enum only if `Formula` is enum as well.
Serializable enum may omit some (or all) variants from `Formula`.
It may not have variants missing in `Formula`.
Each variant then follows rules for structures.

For convenience `Infallible` implements `Serialize` for enum formulas.

### Deserialize

`Deserialize<'de, Formula>` trait is used to implement deserialization 
according to a specific formula.
Deserialization reads from bytes slice constructs deserialized value.
Deserialization *should not* perform dynamic allocations except those
that required to construct and initialize deserialized value.
E.g. it is allowed to allocate when `Vec<T>` is produced if non-zero
number of `T` values are deserialized. It *should not* over-allocate.

Similar to `Serialize` *alkahest* provides a number of out-of-the-box
implementations of `Deserialize` trait.
`From<T>` types can be deserialized with primitive formula `T`.

Values that can be deserialized with formula `F`
can also deserialize with `Ref<F>`, it reads address and length
and proceeds with formula `F`.

`Vec<T>` may deserialize with slice formula.
`Deserialize<'de, [F]>` is implemented for `alkahest::SliceIter<'de, T>` type
that implements `Iterator` and lazily deserialize elements of type
`T: Deserialize<'de, F>`. `SliceIter` is cloneable,
can be iterated from both ends and skips elements for in constant time.
For convenience `SliceIter` also deserializes with array formula.

Deriving `Deserialize` for a type will generate `Deserialize` implementation,
formula is specified in attribute `#[alkahest(FormulaRef)]` or
`#[alkahest(deserialize(FormulaRef))]`. `FormulaRef` is typically a type.
When generics are used it also contains generic parameters and bounds.
If formula is not specified - `Self` is assumed.
`Formula` should be derived for the type as well.
It is in-advised to derive `Deserialize` for formulas with
manual `Formula` implementation,
`Deserialize` derive macro generates code that uses non-public items
generated by `Formula` derive macro.
So either both *should have* manual implementation or both derived.

## Interoperability with `serde`

*Alkahest* is cool but `serde` is almost universally used, and for good reasons.
While designing a `Formula` it may be desireable to include existing type
that supports serialization `serde`, especially if it comes from another crate.
This crate provides `Bincode` and `Bincoded<T>` formulas to cover this.
Anything with `serde::Serialize` implementation can be serialized with `Bincode`
formula, naturally it will be serialized using `bincode` crate.
`Bincoded<T>` is a restricted version of `Bincode` that works only for `T`.

# Usage example

```rust
// This requires two default features - "alloc" and "derive".
#[cfg(all(feature = "derive", feature = "alloc"))]
fn main() {
  use alkahest::{alkahest, serialize_to_vec, deserialize};

  // Define simple formula. Make it self-serializable.
  #[derive(Clone, Debug, PartialEq, Eq)]
  #[alkahest(Formula, SerializeRef, Deserialize)]
  struct MyDataType {
    a: u32,
    b: Vec<u8>,
  }

  // Prepare data to serialize.
  let value = MyDataType {
    a: 1,
    b: vec![2, 3],
  };

  // Use infallible serialization to `Vec`.
  let mut data = Vec::new();

  // Note that this value can be serialized by reference.
  // This is default behavior for `Serialized` derive macro.
  // Some types required ownership transfer for serialization.
  // Notable example is iterators.
  let (size, _) = serialize_to_vec::<MyDataType, _>(&value, &mut data);

  let de = deserialize::<MyDataType, MyDataType>(&data[..size]).unwrap();
  assert_eq!(de, value);
}

#[cfg(not(all(feature = "derive", feature = "alloc")))]
fn main() {}
```

# Benchmarking

Alkahest comes with a benchmark to test against other popular serialization crates.
Simply run `cargo bench --all-features` to see results.

## License

Licensed under either of

* Apache License, Version 2.0, ([license/APACHE](./license/APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
* MIT license ([license/MIT](./license/MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contributions

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
