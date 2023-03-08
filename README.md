# Alkahest - Fantastic serialization library.

[![crates](https://img.shields.io/crates/v/alkahest.svg?style=for-the-badge&label=alkahest)](https://crates.io/crates/alkahest)
[![docs](https://img.shields.io/badge/docs.rs-alkahest-66c2a5?style=for-the-badge&labelColor=555555&logoColor=white)](https://docs.rs/alkahest)
[![actions](https://img.shields.io/github/actions/workflow/status/zakarumych/alkahest/badge.yml?branch=main&style=for-the-badge)](https://github.com/zakarumych/alkahest/actions/workflows/badge.yml)
[![MIT/Apache](https://img.shields.io/badge/license-MIT%2FApache-blue.svg?style=for-the-badge)](./COPYING)
![loc](https://img.shields.io/tokei/lines/github/zakarumych/alkahest?style=for-the-badge)

*Alkahest* is serialization library aimed for packet writing and reading in hot path.
For this purpose *Alkahest* avoids allocations and reads data only on demand.

Key differences of *Alkahest* from other popular serialization crates is zero-overhead serialization and zero-copy lazy deserialization.\
For example to serialize value sequence it is not necessary to construct expensive type with allocations such as vectors.\
Instead sequences are serialized directly from iterators. On deserialization an iterator is returned to the user, which does not parse any element before it is requested.
Which means that data that is not accessed - not parsed either.

*Alkahest* works similarly to *FlatBuffers*,\
but does not require using another language for data scheme definition and running external tool,\
and supports generic schemas.

## Alkahest is very early in development.

If some feature is missing, feel free to create and issue and describe what should be added.

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
