# Alkahest - Fantastic serialization library.

[![crates](https://img.shields.io/crates/v/alkahest.svg?style=for-the-badge&label=alkahest)](https://crates.io/crates/alkahest)
[![docs](https://img.shields.io/badge/docs.rs-alkahest-66c2a5?style=for-the-badge&labelColor=555555&logoColor=white)](https://docs.rs/alkahest)
[![actions](https://img.shields.io/github/workflow/status/zakarumych/alkahest/badge/master?style=for-the-badge)](https://github.com/zakarumych/alkahest/actions?query=workflow%3ARust)
[![MIT/Apache](https://img.shields.io/badge/license-MIT%2FApache-blue.svg?style=for-the-badge)](COPYING)
![loc](https://img.shields.io/tokei/lines/github/zakarumych/alkahest?style=for-the-badge)

*Alkahest* is blazing-fast, zero-deps, zero-overhead, zero-unsafe schema-based serialization library.

# Schema and Serialize traits.

`Schema` trait is used to define types to serve as data schemas.
The esiest way to define new schema is to derive `Schema` trait for a type.
It can be derived for both structs and enums, but no unions. Generics are supported.
The only constrain is that all fields must also implement `Schema`.
User should use trait bounds to ensure that field types with generics implement `Schema`.

`Serialize<Schema>` trait is used to implement serialization according to a schema.
Deriving `Schema` for a `UserType` will generate types with `Serialize<UserType>` implementation.

Primitives like `bool` and integer types implement both `Schema` and can be serlalized from anything that implements `Borrow<PimitiveType>`
`Option<T>` implements `Schema` if `T: Schema` and `Option<U>` implments `Serialize<Option<T>>` if `U: Serialize<T>`.
There's also three ouf-of-the-box schema types:
  * `Seq<T>` - defines a schema as sequence of schemas `T`.
    Any `IntoIterator` type can be used to serialize into `Seq<T>` whenever item type can be serialized to `T`.
    This is major difference from other popular serialization libraries where collection types are used.
    With `Seq<T>` there's no need to allocate a collection and put values for serialization there.
  * `Bytes` - sequence of bytes. Similar to `Seq<u8>`, but can be accessed directly.
  * `Str` - sequence of bytes that is also valud UTF-8 string. Can be accessed as `str`.

# Benchmarking

Alkahest comes with a benchmark to test against other popular serialization crates.
Simply run `cargo bench --all-features` to see results.

## License

Licensed under either of

* Apache License, Version 2.0, ([license/APACHE](license/APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([license/MIT](license/MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributions

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
