#![cfg(feature = "proc")]

use alkahest::{
    Deserialize, DeserializeError, Deserializer, ExactSize, Formula, Mixture, Schema, SchemaKind,
    Serialize, Serializer, Sizes, small,
};

#[derive(Debug, PartialEq)]
struct Leaf(u16);

macro_rules! leaf_revision {
    ($version:literal, $wire:ty, $size:literal) => {
        impl Formula<$version> for Leaf {
            type StackSize<const N: usize> = ExactSize<$size>;
            type HeapSize<const N: usize> = ExactSize<0>;
            const INHABITED: bool = true;
            const SCHEMA: Schema = Schema::new::<Self, $version>(SchemaKind::Primitive(
                alkahest::Primitive::Unsigned($size),
            ));
        }
        impl Serialize<Leaf, $version> for Leaf {
            fn serialize<S: Serializer>(&self, mut serializer: S) -> Result<(), S::Error> {
                serializer.write_direct::<$wire, _>(&(self.0 as $wire))
            }
            fn size_hint<const N: usize>(&self) -> Option<Sizes> {
                Some(Sizes::with_stack($size))
            }
        }
        impl<'de> Deserialize<'de, Leaf, $version> for Leaf {
            fn deserialize<D: Deserializer<'de>>(
                mut deserializer: D,
            ) -> Result<Self, DeserializeError> {
                deserializer
                    .read_value::<$wire, $wire>()
                    .map(|value| Leaf(value as u16))
            }
            fn deserialize_in_place<D: Deserializer<'de>>(
                &mut self,
                deserializer: D,
            ) -> Result<(), DeserializeError> {
                *self = <Self as Deserialize<'de, Leaf, $version>>::deserialize(deserializer)?;
                Ok(())
            }
        }
    };
}
leaf_revision!(0, u8, 1);
leaf_revision!(1, u16, 2);

#[derive(Formula, Serialize, Deserialize, Debug, PartialEq)]
#[alkahest(version = 7)]
struct Record {
    #[alkahest(version = 1)]
    leaf: Leaf,
    tail: u8,
}

#[derive(Mixture, Debug, PartialEq)]
struct DefaultRecord {
    leaf: Leaf,
    tail: u8,
}

#[alkahest::alkahest(Mixture, version = 4)]
#[derive(Debug, PartialEq)]
struct Tuple(#[alkahest(version = 1)] Leaf, u8);

#[derive(Mixture, Debug, PartialEq)]
#[alkahest(version = 2)]
struct Unit;

#[allow(dead_code)]
#[derive(Formula)]
#[alkahest(version = 9)]
enum EnumFormula {
    Unit,
    Tuple(#[alkahest(version = 1)] Leaf),
    Record {
        #[alkahest(version = 1)]
        leaf: Leaf,
        tail: u8,
    },
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[alkahest(EnumFormula, version = 9)]
enum EnumData {
    Unit,
    Tuple(Leaf),
    Record { leaf: Leaf, tail: u8 },
}

#[allow(dead_code)]
#[derive(Formula)]
#[alkahest(version = 6)]
struct BorrowFormula {
    #[alkahest(version = 1)]
    leaf: Leaf,
    text: alkahest::Str,
}

#[alkahest::alkahest(Serialize<BorrowFormula>, version = 6)]
#[alkahest::alkahest(Deserialize<'de, BorrowFormula>, version = 6)]
#[derive(Debug, PartialEq)]
struct Borrowed<'de> {
    leaf: Leaf,
    text: &'de str,
}

#[derive(Mixture, Debug, PartialEq)]
#[alkahest(version = 3)]
struct OnlyNonzero(u16);

#[allow(dead_code)]
#[derive(Formula)]
#[alkahest(version = 8)]
struct GenericFormula<T> {
    #[alkahest(version = 3)]
    value: T,
}

#[alkahest::alkahest(for<F: alkahest::Element<3>> Serialize<GenericFormula<F>>, version = 8 where T: Serialize<F::Formula, 3>)]
#[alkahest::alkahest(for<'de, F: alkahest::Element<3>> Deserialize<'de, GenericFormula<F>>, version = 8 where T: Deserialize<'de, F::Formula, 3>)]
#[derive(Debug, PartialEq)]
struct GenericData<T> {
    value: T,
}

fn round_trip<
    F: Formula<V>,
    T: Serialize<F, V> + for<'de> Deserialize<'de, F, V> + PartialEq + std::fmt::Debug,
    const V: u32,
>(
    value: &T,
) -> Vec<u8> {
    let size = small::serialized_size_at_version::<F, _, V>(value);
    let mut bytes = vec![0; size];
    assert_eq!(
        small::serialize_at_version::<F, _, V>(value, &mut bytes),
        Ok(size)
    );
    let mut decoded = small::deserialize_at_version::<F, T, V>(&bytes).unwrap();
    assert_eq!(&decoded, value);
    small::deserialize_in_place_at_version::<F, _, V>(&mut decoded, &bytes).unwrap();
    assert_eq!(&decoded, value);
    bytes
}

#[test]
fn root_and_child_versions_drive_actual_bytes_and_metadata() {
    let record = Record {
        leaf: Leaf(0x34),
        tail: 0x56,
    };
    assert_eq!(round_trip::<Record, _, 7>(&record), [0x34, 0, 0x56]);
    assert_eq!(
        round_trip::<DefaultRecord, _, 0>(&DefaultRecord {
            leaf: Leaf(0x34),
            tail: 0x56
        }),
        [0x34, 0x56]
    );
    let root = alkahest::schema::<Record, 7>();
    assert_eq!(root.version, 7);
    let SchemaKind::Record(fields) = root.kind else {
        panic!("record metadata")
    };
    assert_eq!((fields[0].element.schema)().version, 1);
    assert_eq!((fields[1].element.schema)().version, 0);
    assert!(
        alkahest::verify_compatibility(root, alkahest::schema::<DefaultRecord, 0>(), 1).is_err()
    );
}

#[test]
fn tuple_unit_and_enum_use_selected_revision() {
    assert_eq!(
        round_trip::<Tuple, _, 4>(&Tuple(Leaf(0x1234), 0x56)),
        [0x34, 0x12, 0x56]
    );
    assert!(round_trip::<Unit, _, 2>(&Unit).is_empty());
    for value in [
        EnumData::Unit,
        EnumData::Tuple(Leaf(0x1234)),
        EnumData::Record {
            leaf: Leaf(0x1234),
            tail: 0x56,
        },
    ] {
        round_trip::<EnumFormula, _, 9>(&value);
    }
}

#[test]
fn separate_borrowed_codecs_resolve_formula_field_versions() {
    let value = Borrowed {
        leaf: Leaf(0x1234),
        text: "versioned",
    };
    let mut bytes = vec![0; 32];
    let len = small::serialize_at_version::<BorrowFormula, _, 6>(&value, &mut bytes).unwrap();
    let decoded =
        small::deserialize_at_version::<BorrowFormula, Borrowed<'_>, 6>(&bytes[..len]).unwrap();
    assert_eq!(decoded, value);
    assert!(bytes[..len].as_ptr_range().contains(&decoded.text.as_ptr()));
}

#[test]
fn generic_formula_accepts_child_without_version_zero() {
    let value = GenericData {
        value: OnlyNonzero(0x1234),
    };
    assert_eq!(
        round_trip::<GenericFormula<OnlyNonzero>, _, 8>(&value),
        [0x34, 0x12]
    );
}

#[derive(Mixture, Debug, PartialEq)]
#[alkahest(version = 8)]
struct GenericMixture<T>(#[alkahest(version = 3)] T);

#[test]
fn mixture_infers_nonzero_child_generic_bounds() {
    let value = GenericMixture(OnlyNonzero(0x1234));
    assert_eq!(
        round_trip::<GenericMixture<OnlyNonzero>, _, 8>(&value),
        [0x34, 0x12]
    );
}

#[derive(Serialize)]
#[alkahest(EnumFormula @Tuple, version = 9)]
struct VariantData(Leaf);

#[test]
fn separate_variant_serializer_resolves_child_revision() {
    let value = VariantData(Leaf(0x1234));
    let mut bytes = [0; 3];
    assert_eq!(
        small::serialize_at_version::<EnumFormula, _, 9>(&value, &mut bytes),
        Ok(3)
    );
    assert_eq!(
        small::deserialize_at_version::<EnumFormula, EnumData, 9>(&bytes).unwrap(),
        EnumData::Tuple(Leaf(0x1234))
    );
}

#[alkahest::alkahest(Serialize<Self>, version = 5)]
#[alkahest::alkahest(Deserialize<Self>, version = 5)]
#[alkahest::alkahest(Formula, version = 5)]
#[derive(Debug, PartialEq)]
struct StackedAttributes(#[alkahest(version = 1)] Leaf);

#[test]
fn stacked_attribute_macros_preserve_formula_pins() {
    assert_eq!(
        round_trip::<StackedAttributes, _, 5>(&StackedAttributes(Leaf(0x1234))),
        [0x34, 0x12]
    );
}

#[derive(Mixture, Debug, PartialEq)]
#[alkahest(version = 1, versions(2 = CurrentEnum))]
enum OldEnum {
    Value,
}

#[derive(Formula, Serialize, Deserialize, Debug, PartialEq)]
#[alkahest(version = 2, versions(1 = OldEnum, 3 = NarrowEnum))]
enum CurrentEnum {
    Value,
    Extra,
}

#[allow(dead_code)]
#[derive(Formula)]
#[alkahest(version = 12, versions(8 = GenericFormula<T>))]
struct LinkedGeneric<T> {
    #[alkahest(version = 3)]
    value: T,
}

#[test]
fn revision_tables_include_local_revision_and_linked_generic_schemas() {
    let entries = <CurrentEnum as Formula<2>>::VERSIONS;
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0]().version, 2);
    assert_eq!(entries[1]().version, 1);
    let entries = <LinkedGeneric<OnlyNonzero> as Formula<12>>::VERSIONS;
    assert_eq!(entries[0]().version, 12);
    assert_eq!(entries[1]().version, 8);
    let SchemaKind::Record(fields) = entries[1]().kind else {
        panic!("record metadata")
    };
    assert_eq!((fields[0].element.schema)().version, 3);
}

#[alkahest::alkahest(Mixture, version = 3, versions(2 = CurrentEnum))]
#[derive(Debug, PartialEq)]
enum NarrowEnum {
    Value,
}

#[test]
fn runtime_enum_compatibility_checks_read_and_write_direction() {
    let mut bytes = [0xaa; 16];
    let len = small::serialize_versioned::<OldEnum, _, 1>(&OldEnum::Value, 1, &mut bytes).unwrap();
    assert_eq!(
        small::deserialize_versioned::<CurrentEnum, CurrentEnum, 2>(&bytes[..len], 1).unwrap(),
        CurrentEnum::Value
    );

    let len = small::serialize_versioned::<CurrentEnum, _, 2>(&CurrentEnum::Value, 2, &mut bytes)
        .unwrap();
    assert!(matches!(
        small::deserialize_versioned::<OldEnum, OldEnum, 1>(&bytes[..len], 2),
        Err(alkahest::VersionError::Incompatible { .. })
    ));
    bytes.fill(0xaa);
    assert!(matches!(
        small::serialize_versioned::<CurrentEnum, _, 2>(&CurrentEnum::Value, 1, &mut bytes),
        Err(alkahest::VersionError::Incompatible { .. })
    ));
    assert_eq!(bytes, [0xaa; 16]);

    let len =
        small::serialize_versioned::<NarrowEnum, _, 3>(&NarrowEnum::Value, 3, &mut bytes).unwrap();
    assert_eq!(
        small::deserialize_versioned::<CurrentEnum, CurrentEnum, 2>(&bytes[..len], 3).unwrap(),
        CurrentEnum::Value
    );
    assert!(matches!(
        small::serialize_versioned::<CurrentEnum, _, 2>(&CurrentEnum::Value, 3, &mut bytes),
        Err(alkahest::VersionError::Incompatible { .. })
    ));
}

#[test]
fn packet_carries_selected_older_version_and_consumes_one_frame() {
    let mut bytes = [0xaa; 32];
    let len = small::pack_versioned::<NarrowEnum, _, 3>(&NarrowEnum::Value, 2, &mut bytes).unwrap();
    assert_eq!(len, 6);
    assert_eq!(&bytes[..len], &[2, 0, 0, 0, 1, 0]);
    let (value, consumed) = small::unpack_versioned::<CurrentEnum, CurrentEnum, 2>(&bytes).unwrap();
    assert_eq!(value, CurrentEnum::Value);
    assert_eq!(consumed, len);
    assert_eq!(bytes[len], 0xaa);
    assert!(matches!(
        small::unpack_versioned::<CurrentEnum, CurrentEnum, 2>(&bytes[..3]),
        Err(alkahest::VersionError::Truncated { .. })
    ));
    bytes[..4].copy_from_slice(&99u32.to_le_bytes());
    assert!(matches!(
        small::unpack_versioned::<CurrentEnum, CurrentEnum, 2>(&bytes),
        Err(alkahest::VersionError::UnknownVersion(99))
    ));
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[alkahest(LinkedGeneric<OnlyNonzero>, version = 12)]
struct LinkedData {
    value: OnlyNonzero,
}

#[test]
fn runtime_root_lookup_preserves_pinned_nested_revision() {
    let value = LinkedData {
        value: OnlyNonzero(0x1234),
    };
    let mut bytes = [0; 16];
    let len =
        small::pack_versioned::<LinkedGeneric<OnlyNonzero>, _, 12>(&value, 8, &mut bytes).unwrap();
    assert_eq!(&bytes[..len], &[8, 0, 0, 0, 2, 0x34, 0x12]);
    let (decoded, consumed) =
        small::unpack_versioned::<LinkedGeneric<OnlyNonzero>, LinkedData, 12>(&bytes[..len])
            .unwrap();
    assert_eq!(decoded, value);
    assert_eq!(consumed, len);
}

#[test]
fn runtime_unknown_version_rejects_before_touching_output() {
    let mut bytes = [0xaa; 16];
    assert!(matches!(
        small::serialize_versioned::<CurrentEnum, _, 2>(&CurrentEnum::Value, 99, &mut bytes),
        Err(alkahest::VersionError::UnknownVersion(99))
    ));
    assert_eq!(bytes, [0xaa; 16]);
    assert!(matches!(
        small::pack_versioned::<CurrentEnum, _, 2>(&CurrentEnum::Value, 99, &mut bytes),
        Err(alkahest::VersionError::UnknownVersion(99))
    ));
    assert_eq!(bytes, [0xaa; 16]);
}

#[allow(dead_code)]
#[derive(Formula)]
#[alkahest(version = 4)]
struct OldBorrowPacketFormula {
    text: alkahest::Indirect<alkahest::Str>,
    #[alkahest(version = 1)]
    leaf: Leaf,
}

#[allow(dead_code)]
#[derive(Formula)]
#[alkahest(version = 6, versions(4 = OldBorrowPacketFormula))]
struct BorrowPacketFormula {
    text: alkahest::Indirect<alkahest::Str>,
    #[alkahest(version = 1)]
    leaf: Leaf,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[alkahest(BorrowPacketFormula, version = 6)]
struct BorrowPacket<'de> {
    text: &'de str,
    leaf: Leaf,
}

#[test]
fn versioned_packet_keeps_borrowed_indirect_addresses_payload_relative() {
    fn check<const WIDTH: usize>() {
        let value = BorrowPacket {
            text: "borrowed",
            leaf: Leaf(0x1234),
        };
        let mut bytes = [0xaa; 128];
        let len = alkahest::manual_size::pack_versioned::<BorrowPacketFormula, _, WIDTH, 6>(
            &value, 4, &mut bytes,
        )
        .unwrap();
        assert_eq!(&bytes[..4], &4u32.to_le_bytes());
        let (decoded, consumed) = alkahest::manual_size::unpack_versioned::<
            BorrowPacketFormula,
            BorrowPacket<'_>,
            WIDTH,
            6,
        >(&bytes)
        .unwrap();
        assert_eq!(decoded, value);
        assert_eq!(consumed, len);
        assert!(
            bytes[4 + WIDTH..len]
                .as_ptr_range()
                .contains(&decoded.text.as_ptr())
        );
        assert_eq!(bytes[len], 0xaa);
    }
    check::<1>();
    check::<2>();
    check::<4>();
    check::<8>();
    check::<16>();
}
