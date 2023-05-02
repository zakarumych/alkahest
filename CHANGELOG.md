# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0]

### Fixed

* Fix recursive types.

### Added

* Custom attribute to derive `Formula`, `Serialize` and `Deserialize`
  when additional data is needed for deriving.

### Changed

* Derive macros do not accept attributes anymore.
  Use custom attribute instead.
* Support manual generic bounds for formula deriving macro.
* Move packet writing into separate trait.

## [0.2.0]

* Reimplement with no unsafe code.
* Fuse deserialization with cheap direct access and lazy deserialization
  using `Lazy` and `DeIter`.
* Add support for unsized formulas.
* Derive macro doesn't generate new types anymore.
* Derive `Serialize` for a type to serialize it into specified formula,
  generated code will check that type is compatible with formula,
  works only for formulas implemented using derive macro.
* Derive `Deserialize` for a type to deserialize it from specified formula.
  generated code will check that type is compatible with formula,
  works only for formulas implemented using derive macro.
* Interoperability with `serde` using `bincode`
* Different flavors of `serialize` methods.
  Fallible, panicking, with growing buffer,
  with exact size calculation on fail.

## [0.1.0] - 2021-07-20

Implemented writing and reading packets with typed formula.
Implement formulas for primitives and sequences.
Implement proc-macro to derive formulas for structures and enums.
