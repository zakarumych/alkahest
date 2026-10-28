use alkahest_core::{
    Deserialize, DeserializeError, Deserializer, Element, ExactSize, Field, Formula, Indirect,
    Schema, SchemaKind, Serialize, Serializer, Sizes, Variant, VersionError, manual_size, schema,
};

struct Contract;

const SHORT: &[Variant] = &[
    Variant {
        name: "A",
        tag: 0,
        fields: &[],
    },
    Variant {
        name: "B",
        tag: 1,
        fields: &[],
    },
];
const WIDE: &[Variant] = &[
    Variant {
        name: "A",
        tag: 0,
        fields: &[],
    },
    Variant {
        name: "B",
        tag: 1,
        fields: &[],
    },
    Variant {
        name: "C",
        tag: 2,
        fields: &[],
    },
];

macro_rules! contract {
    ($version:literal, $variants:ident) => {
        impl Formula<$version> for Contract {
            type StackSize<const WIDTH: usize> = ExactSize<1>;
            type HeapSize<const WIDTH: usize> = ExactSize<0>;
            const INHABITED: bool = true;
            const SCHEMA: Schema = Schema::new::<Self, $version>(SchemaKind::Enum {
                variants: $variants,
                discriminant_count: $variants.len(),
            });
            const VERSIONS: &'static [fn() -> &'static Schema] =
                &[schema::<Self, 1>, schema::<Self, 2>, schema::<Self, 3>];
        }
    };
}
contract!(1, SHORT);
contract!(2, WIDE);
contract!(3, SHORT);

#[derive(Debug, PartialEq, Eq)]
struct Value(u8);

impl<const V: u32> Serialize<Contract, V> for Value
where
    Contract: Formula<V>,
{
    fn serialize<S: Serializer>(&self, mut serializer: S) -> Result<(), S::Error> {
        assert!(self.0 < if V == 2 { 3 } else { 2 });
        serializer.write_bytes(&[self.0])
    }
    fn size_hint<const WIDTH: usize>(&self) -> Option<Sizes> {
        Some(Sizes::with_stack(1))
    }
}

impl<'de, const V: u32> Deserialize<'de, Contract, V> for Value
where
    Contract: Formula<V>,
{
    fn deserialize<D: Deserializer<'de>>(mut de: D) -> Result<Self, DeserializeError> {
        let tag = de.read_byte()?;
        if tag < if V == 2 { 3 } else { 2 } {
            Ok(Self(tag))
        } else {
            Err(DeserializeError::WrongVariant(tag.into()))
        }
    }
    fn deserialize_in_place<D: Deserializer<'de>>(
        &mut self,
        de: D,
    ) -> Result<(), DeserializeError> {
        *self = <Self as Deserialize<Contract, V>>::deserialize(de)?;
        Ok(())
    }
}

#[test]
fn runtime_write_can_target_older_compatible_revision() {
    let mut bytes = [0xcc; 8];
    let len =
        alkahest_core::serialize_versioned::<Contract, _, 3>(&Value(1), 1, &mut bytes).unwrap();
    assert_eq!(len, 1);
    assert_eq!(bytes, [1, 0xcc, 0xcc, 0xcc, 0xcc, 0xcc, 0xcc, 0xcc]);
    assert_eq!(
        alkahest_core::deserialize_versioned::<Contract, Value, 1>(&bytes[..len], 1).unwrap(),
        Value(1)
    );
    assert_eq!(
        alkahest_core::serialized_size_versioned::<Contract, _, 3>(&Value(1), 1).unwrap(),
        len
    );
}

#[test]
fn runtime_reader_acceptance_is_directional() {
    let mut bytes = [0; 4];
    let len =
        alkahest_core::serialize_versioned::<Contract, _, 1>(&Value(1), 1, &mut bytes).unwrap();
    assert_eq!(
        alkahest_core::deserialize_versioned::<Contract, Value, 2>(&bytes[..len], 1).unwrap(),
        Value(1)
    );
    assert!(matches!(
        alkahest_core::deserialize_versioned::<Contract, Value, 1>(&bytes[..len], 2),
        Err(VersionError::Incompatible {
            local_version: 1,
            wire_version: 2,
            ..
        })
    ));
    // The target revision accepts every value in the local writer's domain.
    assert!(alkahest_core::serialize_versioned::<Contract, _, 1>(&Value(1), 2, &mut bytes).is_ok());
}

#[test]
fn incompatible_operations_fail_before_modification_even_for_shared_value() {
    let mut bytes = [0x55; 32];
    assert!(matches!(
        alkahest_core::serialize_versioned::<Contract, _, 2>(&Value(0), 1, &mut bytes),
        Err(VersionError::Incompatible { .. })
    ));
    assert!(matches!(
        alkahest_core::pack_versioned::<Contract, _, 2>(&Value(0), 1, &mut bytes),
        Err(VersionError::Incompatible { .. })
    ));
    assert_eq!(bytes, [0x55; 32]);
    let mut place = Value(1);
    assert!(matches!(
        alkahest_core::deserialize_in_place_versioned::<Contract, _, 1>(&mut place, &[0], 2),
        Err(VersionError::Incompatible { .. })
    ));
    assert_eq!(place, Value(1));
}

#[test]
fn unknown_versions_fail_before_modification() {
    let mut bytes = [0x55; 32];
    assert!(matches!(
        alkahest_core::serialize_versioned::<Contract, _, 2>(&Value(0), 99, &mut bytes),
        Err(VersionError::UnknownVersion(99))
    ));
    assert!(matches!(
        alkahest_core::deserialize_versioned::<Contract, Value, 2>(&[0], 99),
        Err(VersionError::UnknownVersion(99))
    ));
    assert_eq!(bytes, [0x55; 32]);
}

struct Duplicate;
struct Missing;
struct Opaque;
struct Wrong;
struct Normalized;

macro_rules! custom {
    ($name:ident, $version:literal, $table:expr) => {
        impl Formula<5> for $name {
            type StackSize<const WIDTH: usize> = ExactSize<1>;
            type HeapSize<const WIDTH: usize> = ExactSize<0>;
            const INHABITED: bool = true;
            const SCHEMA: Schema = Schema {
                version: $version,
                ..Schema::unknown::<Self, 5>()
            };
            const VERSIONS: &'static [fn() -> &'static Schema] = $table;
        }
        impl Serialize<$name, 5> for Value {
            fn serialize<S: Serializer>(&self, mut serializer: S) -> Result<(), S::Error> {
                serializer.write_bytes(&[self.0])
            }
            fn size_hint<const WIDTH: usize>(&self) -> Option<Sizes> {
                Some(Sizes::with_stack(1))
            }
        }
    };
}
custom!(Duplicate, 5, &[schema::<Self, 5>, schema::<Self, 5>]);
custom!(Missing, 5, &[schema::<Contract, 1>]);
custom!(Opaque, 5, &[schema::<Self, 5>, schema::<Contract, 1>]);
custom!(Wrong, 6, &[schema::<Self, 5>]);
custom!(Normalized, 5, &[schema::<u32, 5>]);

#[test]
fn static_tables_are_validated_and_local_schema_is_authoritative() {
    let mut bytes = [0x55; 8];
    assert!(matches!(
        alkahest_core::serialize_versioned::<Duplicate, _, 5>(&Value(0), 5, &mut bytes),
        Err(VersionError::DuplicateVersion(5))
    ));
    assert!(matches!(
        alkahest_core::serialize_versioned::<Missing, _, 5>(&Value(0), 5, &mut bytes),
        Err(VersionError::MissingLocalVersion(5))
    ));
    assert!(matches!(
        alkahest_core::serialize_versioned::<Wrong, _, 5>(&Value(0), 5, &mut bytes),
        Err(VersionError::InvalidSchema)
    ));
    assert_eq!(bytes, [0x55; 8]);
    assert_eq!(
        alkahest_core::serialize_versioned::<Normalized, _, 5>(&Value(1), 5, &mut bytes).unwrap(),
        1
    );
}

#[test]
fn opaque_local_codec_allowed_but_cross_version_requires_proof() {
    let mut bytes = [0x55; 8];
    assert_eq!(
        alkahest_core::serialize_versioned::<Opaque, _, 5>(&Value(1), 5, &mut bytes).unwrap(),
        1
    );
    assert!(matches!(
        alkahest_core::serialize_versioned::<Opaque, _, 5>(&Value(0), 1, &mut bytes),
        Err(VersionError::Incompatible {
            reason: alkahest_core::CompatibilityError::UnknownSchema,
            ..
        })
    ));
}

fn packet_roundtrip<const WIDTH: usize>() {
    let mut bytes = [0xcc; 80];
    let len =
        manual_size::pack_versioned::<Contract, _, WIDTH, 3>(&Value(1), 1, &mut bytes).unwrap();
    assert_eq!(len, 5 + WIDTH);
    assert_eq!(&bytes[..4], &1u32.to_le_bytes());
    assert_eq!(&bytes[4..4 + WIDTH], &1u128.to_le_bytes()[..WIDTH]);
    assert_eq!(bytes[4 + WIDTH], 1);
    assert_eq!(
        manual_size::pack_size_versioned::<Contract, _, WIDTH, 3>(&Value(1), 1).unwrap(),
        len
    );
    let (value, consumed) =
        manual_size::unpack_versioned::<Contract, Value, WIDTH, 2>(&bytes).unwrap();
    assert_eq!(value, Value(1));
    assert_eq!(consumed, len);
    let mut place = Value(0);
    assert_eq!(
        manual_size::unpack_in_place_versioned::<Contract, _, WIDTH, 2>(&mut place, &bytes)
            .unwrap(),
        len
    );
    assert_eq!(place, Value(1));
}

#[test]
fn packets_roundtrip_all_supported_address_widths() {
    packet_roundtrip::<1>();
    packet_roundtrip::<2>();
    packet_roundtrip::<3>();
    packet_roundtrip::<4>();
    packet_roundtrip::<5>();
    packet_roundtrip::<6>();
    packet_roundtrip::<7>();
    packet_roundtrip::<8>();
    packet_roundtrip::<9>();
    packet_roundtrip::<10>();
    packet_roundtrip::<11>();
    packet_roundtrip::<12>();
    packet_roundtrip::<13>();
    packet_roundtrip::<14>();
    packet_roundtrip::<15>();
    packet_roundtrip::<16>();
}

#[test]
fn frames_allow_concatenation() {
    let mut bytes = [0; 64];
    let first = alkahest_core::pack_versioned::<Contract, _, 3>(&Value(1), 1, &mut bytes).unwrap();
    let second =
        alkahest_core::pack_versioned::<Contract, _, 2>(&Value(2), 2, &mut bytes[first..]).unwrap();
    let (a, n) =
        alkahest_core::unpack_versioned::<Contract, Value, 2>(&bytes[..first + second]).unwrap();
    let (b, m) =
        alkahest_core::unpack_versioned::<Contract, Value, 2>(&bytes[n..first + second]).unwrap();
    assert_eq!((a, b), (Value(1), Value(2)));
    assert_eq!((n, m), (first, second));
}

#[test]
fn malformed_packet_lengths_and_unknown_versions_are_errors() {
    for len in 0..8 {
        assert!(matches!(
            alkahest_core::unpack_versioned::<Contract, Value, 2>(&[0; 8][..len]),
            Err(VersionError::Truncated { .. })
        ));
    }
    let mut bytes = [0; 24];
    bytes[..4].copy_from_slice(&1u32.to_le_bytes());
    bytes[4..8].copy_from_slice(&100u32.to_le_bytes());
    assert!(matches!(
        alkahest_core::unpack_versioned::<Contract, Value, 2>(&bytes),
        Err(VersionError::Truncated { .. })
    ));
    bytes[4..20].fill(0xff);
    assert!(matches!(
        manual_size::unpack_versioned::<Contract, Value, 16, 2>(&bytes),
        Err(VersionError::LengthOverflow)
    ));
    bytes[4..20].copy_from_slice(&(usize::MAX as u128).to_le_bytes());
    assert!(matches!(
        manual_size::unpack_versioned::<Contract, Value, 16, 2>(&bytes),
        Err(VersionError::LengthOverflow)
    ));
    bytes.fill(0);
    bytes[..4].copy_from_slice(&99u32.to_le_bytes());
    bytes[4] = 1;
    assert!(matches!(
        alkahest_core::unpack_versioned::<Contract, Value, 2>(&bytes),
        Err(VersionError::UnknownVersion(99))
    ));
}

#[test]
fn invalid_address_widths_return_errors_without_codec_instantiation_panics() {
    let mut bytes = [0x55; 32];
    assert!(matches!(
        manual_size::serialize_versioned::<Contract, _, 0, 1>(&Value(0), 1, &mut bytes),
        Err(VersionError::UnsupportedSizeBytes(0))
    ));
    assert!(matches!(
        manual_size::deserialize_versioned::<Contract, Value, 17, 1>(&bytes, 1),
        Err(VersionError::UnsupportedSizeBytes(17))
    ));
    assert!(matches!(
        manual_size::pack_versioned::<Contract, _, 17, 1>(&Value(0), 1, &mut bytes),
        Err(VersionError::UnsupportedSizeBytes(17))
    ));
    assert!(matches!(
        manual_size::unpack_versioned::<Contract, Value, 0, 1>(&bytes),
        Err(VersionError::UnsupportedSizeBytes(0))
    ));
    assert_eq!(bytes, [0x55; 32]);
}

struct IndirectRoot;
macro_rules! indirect_root {
    ($version:literal) => {
        impl Formula<$version> for IndirectRoot {
            type StackSize<const WIDTH: usize> =
                <(Indirect<Contract>, u8) as Formula<$version>>::StackSize<WIDTH>;
            type HeapSize<const WIDTH: usize> =
                <(Indirect<Contract>, u8) as Formula<$version>>::HeapSize<WIDTH>;
            const INHABITED: bool = true;
            const SCHEMA: Schema = Schema::new::<Self, $version>(SchemaKind::Record(&[
                Field {
                    name: "child",
                    element: <Indirect<Contract> as Element<$version>>::SCHEMA,
                },
                Field {
                    name: "marker",
                    element: <u8 as Element<$version>>::SCHEMA,
                },
            ]));
            const VERSIONS: &'static [fn() -> &'static Schema] =
                &[schema::<Self, 1>, schema::<Self, 2>];
        }
    };
}
indirect_root!(1);
indirect_root!(2);

impl<const V: u32> Serialize<IndirectRoot, V> for (Value, u8)
where
    IndirectRoot: Formula<V>,
    Contract: Formula<V>,
{
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<(), S::Error> {
        <Self as Serialize<(Indirect<Contract>, u8), V>>::serialize(self, serializer)
    }
    fn size_hint<const WIDTH: usize>(&self) -> Option<Sizes> {
        <Self as Serialize<(Indirect<Contract>, u8), V>>::size_hint::<WIDTH>(self)
    }
}
impl<'de, const V: u32> Deserialize<'de, IndirectRoot, V> for (Value, u8)
where
    IndirectRoot: Formula<V>,
    Contract: Formula<V>,
{
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, DeserializeError> {
        <Self as Deserialize<'de, (Indirect<Contract>, u8), V>>::deserialize(de)
    }
    fn deserialize_in_place<D: Deserializer<'de>>(
        &mut self,
        de: D,
    ) -> Result<(), DeserializeError> {
        <Self as Deserialize<'de, (Indirect<Contract>, u8), V>>::deserialize_in_place(self, de)
    }
}

#[test]
fn indirect_addresses_are_relative_to_payload_and_preserve_child_versions() {
    let value = (Value(1), 42u8);
    let mut raw = [0; 32];
    let bare = alkahest_core::serialize_at_version::<IndirectRoot, _, 1>(&value, &mut raw).unwrap();
    let mut bytes = [0xcc; 80];
    let len = alkahest_core::pack_versioned::<IndirectRoot, _, 1>(&value, 2, &mut bytes).unwrap();
    assert_eq!(&bytes[8..len], &raw[..bare]);
    let (decoded, consumed) =
        alkahest_core::unpack_versioned::<IndirectRoot, (Value, u8), 2>(&bytes).unwrap();
    assert_eq!(decoded, value);
    assert_eq!(consumed, len);
    assert_eq!(bytes[len..], [0xcc; 80][len..]);
}

#[test]
fn packet_size_limits_and_short_outputs_leave_output_untouched() {
    let mut bytes = [0xcc; 4];
    assert!(matches!(
        alkahest_core::pack_versioned::<Contract, _, 1>(&Value(1), 1, &mut bytes),
        Err(VersionError::Codec(_))
    ));
    assert_eq!(bytes, [0xcc; 4]);
    let value = [0u8; 256];
    let mut large = [0xcc; 300];
    assert!(matches!(
        manual_size::pack_versioned::<alkahest_core::List<u8, 256, 256>, _, 1, 0>(
            &value, 0, &mut large
        ),
        Err(VersionError::LengthOverflow)
    ));
    assert!(matches!(
        manual_size::pack_versioned::<alkahest_core::List<u8>, _, 1, 0>(&value, 0, &mut large),
        Err(VersionError::LengthOverflow)
    ));
    assert!(matches!(
        manual_size::pack_size_versioned::<alkahest_core::List<u8>, _, 1, 0>(&value, 0),
        Err(VersionError::LengthOverflow)
    ));
    assert_eq!(large, [0xcc; 300]);
}

#[test]
fn exact_payload_length_rejects_hidden_trailing_bytes_before_mutation() {
    let overlong = [0, 0, 0, 0, 3, 42, 99, 88];
    assert!(matches!(
        alkahest_core::small::unpack_versioned::<u8, u8, 0>(&overlong),
        Err(VersionError::PayloadLength {
            expected: 1,
            actual: 3
        })
    ));
    let mut place = 7u8;
    assert!(matches!(
        alkahest_core::small::unpack_in_place_versioned::<u8, _, 0>(&mut place, &overlong),
        Err(VersionError::PayloadLength {
            expected: 1,
            actual: 3
        })
    ));
    assert_eq!(place, 7);
    for input in [&[][..], &[1, 2][..]] {
        assert!(matches!(
            alkahest_core::deserialize_versioned::<u8, u8, 0>(input, 0),
            Err(VersionError::PayloadLength { expected: 1, .. })
        ));
    }
    assert!(matches!(
        alkahest_core::small::unpack_versioned::<(), (), 0>(&[0, 0, 0, 0, 1, 0]),
        Err(VersionError::PayloadLength {
            expected: 0,
            actual: 1
        })
    ));
}

#[test]
fn exact_indirect_frame_length_includes_both_heap_and_stack() {
    let mut bytes = [0; 80];
    let len = alkahest_core::pack_versioned::<IndirectRoot, _, 1>(&((Value(1), 42)), 2, &mut bytes)
        .unwrap();
    let payload = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
    bytes[4..8].copy_from_slice(&(payload + 1).to_le_bytes());
    assert!(
        matches!(alkahest_core::unpack_versioned::<IndirectRoot, (Value, u8), 2>(&bytes[..len + 1]),
        Err(VersionError::PayloadLength { expected, actual }) if expected == payload as usize && actual == payload as usize + 1)
    );
}
