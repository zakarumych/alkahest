# Explicit formula versions

`Formula<VERSION>`, `Serialize<F, VERSION>` and
`Deserialize<'de, F, VERSION>` select a wire contract at compile time. The
version is a `u32`, with `0` as the default. `Element`, `AsFormula`, `Mixture`
and `Component` carry the same parameter. A formula may implement more than
one version; each implementation supplies its own sizes, metadata and codecs.

The local codec version is a compile-time parameter. The wire version can
be selected at runtime using a root formula's static version table. Bare
serialization takes that version explicitly; versioned packets store it in
their header. Root selection resolves child versions from static metadata.

## Deriving a version

```rust
use alkahest::{Formula, Serialize, Deserialize};

#[derive(Formula, Serialize, Deserialize)]
#[alkahest(version = 1)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Formula, Serialize, Deserialize)]
#[alkahest(version = 2)]
struct Entity {
    id: u64,
    #[alkahest(version = 1)]
    position: Position,
}
```

The item annotation selects the root implementation. A field annotation pins
that element's version, including its indirect placement when applicable.
Unannotated fields select version **0**, independently of the root version.
This prevents an unrelated root version increase from changing every child.
Tuple fields and enum payload fields support the same annotation.

For separate data and formula types, use
`#[alkahest(FormulaType, version = 2)]` on the data type's codec derives.
The formula's field declarations determine child versions for these codecs.
The attribute macro also accepts an item version, for example
`#[alkahest::alkahest(Formula, version = 2)]`.

Built-in scalar formulas support every version with the same encoding.
Built-in containers use their selected version for their elements; the
version pinned on a derived field therefore also selects the element version
inside a list, tuple or option at that field.

## Declaring available root versions

Each formula version exposes `Formula<VERSION>::VERSIONS`, a static slice of
schema callbacks. Its default contains only that local version. Derived
formulas can declare other revisions explicitly:

```rust,ignore
#[derive(Formula, Serialize, Deserialize)]
#[alkahest(version = 2, versions(1 = OldEntity))]
struct Entity {
    // Fields of the local version 2 contract.
}
```

The generated table contains the local schema plus `schema::<OldEntity, 1>`.
The root formula types may differ between revisions. The derive rejects
duplicate version numbers and requires each linked formula to implement the
declared version. A manual implementation can supply the same table through
the associated constant. The table maps versions to metadata, not arbitrary
codec callbacks or a global registry.

Runtime selection validates duplicate numbers, the presence of the local
version and agreement of `F::SCHEMA.version` with the local const parameter.
The trait's `F::SCHEMA` is authoritative for the local entry; a table entry
with that version is normalized to this descriptor. Unknown wire versions
return `VersionError::UnknownVersion`.

## Runtime serialization entry points

The `_versioned` entry points select a declared wire version at runtime.
The trailing const parameter selects the local codec:

```rust,ignore
let wire_version = 1;
let size = alkahest::serialized_size_versioned::<Entity, _, 2>(&value, wire_version)?;
let mut bytes = vec![0; size];
alkahest::serialize_versioned::<Entity, _, 2>(&value, wire_version, &mut bytes)?;
let decoded: Entity = alkahest::deserialize_versioned::<Entity, _, 2>(&bytes, wire_version)?;
```

Reads verify that the local reader accepts the declared wire writer. Writes
verify that the requested wire contract accepts the local writer's entire
declared domain. A compatible older wire revision is a valid write target.
The local codec is reused only when this conservative proof succeeds; there
is no migration, per-value downcast, arbitrary revision dispatch or silent
fallback. Even a value shared by both domains is rejected if their required
direction is incompatible. Unknown and incompatible versions fail before
output buffers or in-place destinations are modified.

When wire and local versions are equal, the exact local codec is known and
opaque metadata is permitted. Cross-version operations always require
structural verification, so opaque metadata cannot establish compatibility.

The `small`, `medium`, `large`, `huge` and `humongous` modules expose these
functions with their fixed address/length widths. `manual_size` takes
`SIZE_BYTES` before `LOCAL_VERSION`, supporting widths 1 through 16. The
root re-exports use the four-byte `large` width. In-place decoding has the
same runtime validation. Bare input must contain the payload alone. If the
formula has an exact total size, its payload length is checked before decoding.

## Versioned packets

`pack_versioned(value, wire_version, output)` writes exactly:

```text
[4-byte little-endian wire version]
[SIZE_BYTES-byte little-endian bare payload length]
[bare payload]
```

The payload length excludes the header. All indirect addresses are relative
to the first byte of the payload. The header is present even for exact-sized
formulas. `pack_size_versioned` includes both header fields.

`unpack_versioned(input)` obtains the wire version from this header, validates
compatibility and decodes the exact payload subslice. It returns `(value,
consumed)`, allowing concatenated packets. `unpack_in_place_versioned` returns
the consumed length. Truncated headers or payloads, unsupported widths and
lengths exceeding the platform or selected width return `VersionError`.
Exact-sized formulas reject headers that declare a different payload length,
including extra bytes that their codecs would otherwise leave unread.

## Explicit codec entry points

The existing unsuffixed entry points select version 0. The `_at_version`
entry points select a codec directly through the trailing const parameter:

```rust,ignore
alkahest::serialize_at_version::<Entity, _, 2>(&value, &mut bytes)?;
let decoded: Entity = alkahest::deserialize_at_version::<Entity, _, 2>(&bytes)?;
```

These functions perform no runtime version lookup or compatibility checks.
Their payloads and packet forms do not carry version tags. They are useful
when the caller already guarantees the exact contract. Lazy decoding and
lower-level serializer/deserializer methods also carry explicit const
versions; the lower-level methods retain their `_versioned` names.

## Static metadata

`schema::<F, VERSION>()` returns `&'static Schema`. Records contain ordered
`Field` descriptions; enum variants contain the actual numeric discriminant
and payload fields. An `ElementSchema` contains an indirection flag and a
callback resolving its child's schema. Following these callbacks reveals
the complete selection of child versions without inspecting a value.

Callbacks permit recursive static descriptions without constructing an
infinite constant. No registry, allocation or runtime version map is required.
`Schema::new::<F, VERSION>(kind)` obtains layout through the formula's actual
`StackSize<SIZE_BYTES>` and `HeapSize<SIZE_BYTES>` associated types.
`layout::<F, VERSION>(size_bytes)` exposes the same information directly.

Enum descriptors use the codec's inhabited discriminant count and compressed
numeric tags. An uninhabited variant has tag `usize::MAX` and no wire
representation. Field and variant names are descriptive only; the format
uses field position and numeric variant tags.

## Requiring compatibility

```rust,ignore
use alkahest::{Compatibility, require_compatibility, schema};

require_compatibility(
    schema::<NewFormula, 2>(),
    schema::<OldFormula, 1>(),
    Compatibility::Both,
    4, // Both endpoints use four-byte addresses and lengths.
)?;
```

`Backward` means the new reader accepts the old writer. `Forward` means the
old reader accepts the new writer. `Both` checks both directions. These are
explicit requirements enforced by a returned `Result`; they are not derive
annotations. Call the function from protocol registration, application
startup or a regression test and handle its result.

`verify_compatibility(reader, writer, size_bytes)` checks one direction.
`verify_backward_compatibility(new, old, size_bytes)` and
`verify_forward_compatibility(new, old, size_bytes)` name the directions.

Verification is deliberately conservative:

- Stack/heap bounds, their exact/bounded/unbounded classification and
  inhabitance must agree at each compared node.
- Primitive encoding identities must agree; equal widths do not make signed
  integers, unsigned integers, floating point or booleans interchangeable.
- Records require matching field counts, order, placement and child contracts.
- Every writer enum tag must exist in the reader with a compatible payload,
  and discriminant widths must match. A variant addition can therefore be
  backward compatible when the layout and tag encoding remain unchanged.
- Writer list length ranges must be accepted by the reader. Fixed arrays
  omit a length prefix and cannot be treated as variable lists. Layout
  equality still applies, so some accepted-value widenings remain unverified.
- Options and UTF-8 strings retain their distinct wire encodings.
- Opaque metadata and recursion reaching 64 nested comparisons return errors.
  Pointer equality does not bypass these checks.

An error means compatibility was not established; it does not imply that
every encoded value would fail. The check establishes a declared wire
contract, not application semantics: swapping same-typed fields or reusing
an enum tag for a different meaning cannot be detected from wire shape.

Handwritten `Formula` and `Element` implementations default to opaque
metadata. To participate in checks, an implementation must explicitly
describe its encoding and honor that description. Arbitrary handwritten
`Serialize`/`Deserialize` implementations remain trusted: static metadata
cannot prove what arbitrary Rust code reads or writes.

## Checking versions in CI

The [`alkahest-schema` tool](schema/README.md) exports formula metadata and
compares it with a trusted baseline. It rejects unchanged versions with changed
schemas, including nested element-version changes. Register reusable child
formulas independently so advancing a parent cannot hide a missing child bump.
The checker runs in any CI and includes JSON output, GitHub annotations and a
composite GitHub Action. See its guide for baseline initialization, trusted
baseline selection and a runnable exporter.
