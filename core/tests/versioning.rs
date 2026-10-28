use alkahest_core::{
    Deserialize, DeserializeError, Deserializer, Element, ExactSize, Field, Formula, Lazy, List,
    Primitive, Schema, SchemaKind, Serialize, Serializer, SizeBound, Sizes, manual_size, schema,
    stack_size_versioned,
};

#[cfg(feature = "alloc")]
use alkahest_core::Indirect;

struct Word;

macro_rules! word_version {
    ($version:literal, $wire:ty) => {
        impl Formula<$version> for Word {
            type StackSize<const SIZE_BYTES: usize> = ExactSize<{ size_of::<$wire>() }>;
            type HeapSize<const SIZE_BYTES: usize> = ExactSize<0>;
            const INHABITED: bool = true;
            const SCHEMA: Schema = Schema::new::<Self, $version>(SchemaKind::Primitive(
                Primitive::Unsigned(size_of::<$wire>() as u8),
            ));
        }

        impl Serialize<Word, $version> for Value {
            fn serialize<S: Serializer>(&self, mut serializer: S) -> Result<(), S::Error> {
                let value = <$wire>::try_from(self.0).unwrap();
                serializer.write_direct_versioned::<$wire, _, $version>(&value)
            }

            fn size_hint<const SIZE_BYTES: usize>(&self) -> Option<Sizes> {
                Some(Sizes::with_stack(size_of::<$wire>()))
            }
        }

        impl<'de> Deserialize<'de, Word, $version> for Value {
            fn deserialize<D: Deserializer<'de>>(
                mut deserializer: D,
            ) -> Result<Self, DeserializeError> {
                deserializer
                    .read_value_versioned::<$wire, $wire, $version>()
                    .map(|v| Self(v.into()))
            }

            fn deserialize_in_place<D: Deserializer<'de>>(
                &mut self,
                deserializer: D,
            ) -> Result<(), DeserializeError> {
                *self = <Self as Deserialize<Word, $version>>::deserialize(deserializer)?;
                Ok(())
            }
        }
    };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Value(u16);

word_version!(1, u8);
word_version!(2, u16);

struct Root(Word);

impl Formula<7> for Root {
    type StackSize<const SIZE_BYTES: usize> = <Word as Formula<2>>::StackSize<SIZE_BYTES>;
    type HeapSize<const SIZE_BYTES: usize> = ExactSize<0>;
    const INHABITED: bool = true;
    const SCHEMA: Schema = Schema::new::<Self, 7>(SchemaKind::Record(&[Field {
        name: "word",
        element: <Word as Element<2>>::SCHEMA,
    }]));
}

impl Serialize<Root, 7> for Value {
    fn serialize<S: Serializer>(&self, mut serializer: S) -> Result<(), S::Error> {
        alkahest_core::private::with_element_versioned::<Root, Word, 2>(|f| &f.0)
            .serialize(self, &mut serializer)
    }

    fn size_hint<const SIZE_BYTES: usize>(&self) -> Option<Sizes> {
        alkahest_core::private::with_element_versioned::<Root, Word, 2>(|f| &f.0)
            .size_hint::<_, SIZE_BYTES>(self)
    }
}

impl<'de> Deserialize<'de, Root, 7> for Value {
    fn deserialize<D: Deserializer<'de>>(mut deserializer: D) -> Result<Self, DeserializeError> {
        alkahest_core::private::with_element_versioned::<Root, Word, 2>(|f| &f.0)
            .deserialize(&mut deserializer)
    }

    fn deserialize_in_place<D: Deserializer<'de>>(
        &mut self,
        mut deserializer: D,
    ) -> Result<(), DeserializeError> {
        alkahest_core::private::with_element_versioned::<Root, Word, 2>(|f| &f.0)
            .deserialize_in_place(self, &mut deserializer)
    }
}

fn layout_roundtrip<const SIZE_BYTES: usize>() {
    let mut bytes = [0; 32];
    let value = Value(42);
    let len =
        manual_size::serialize_at_version::<Word, _, SIZE_BYTES, 1>(&value, &mut bytes).unwrap();
    assert_eq!(&bytes[..len], &[42]);
    assert_eq!(
        manual_size::deserialize_at_version::<Word, Value, SIZE_BYTES, 1>(&bytes[..len]).unwrap(),
        value
    );
    let len =
        manual_size::serialize_at_version::<Word, _, SIZE_BYTES, 2>(&value, &mut bytes).unwrap();
    assert_eq!(&bytes[..len], &[42, 0]);
    assert_eq!(
        manual_size::serialized_size_at_version::<Word, _, SIZE_BYTES, 2>(&value),
        2
    );
    assert_eq!(
        manual_size::deserialize_at_version::<Word, Value, SIZE_BYTES, 2>(&bytes[..len]).unwrap(),
        value
    );
    let mut place = Value(999);
    manual_size::deserialize_in_place_at_version::<Word, _, SIZE_BYTES, 2>(
        &mut place,
        &bytes[..len],
    )
    .unwrap();
    assert_eq!(place, value);
    assert_eq!(
        stack_size_versioned::<Word, SIZE_BYTES, 1>(),
        SizeBound::Exact(1)
    );
    assert_eq!(
        stack_size_versioned::<Word, SIZE_BYTES, 2>(),
        SizeBound::Exact(2)
    );
}

#[test]
fn explicit_version_selects_layout_at_every_address_width() {
    layout_roundtrip::<1>();
    layout_roundtrip::<2>();
    layout_roundtrip::<4>();
    layout_roundtrip::<8>();
    layout_roundtrip::<16>();
}

#[test]
fn root_selects_child_version_from_static_metadata() {
    let root = schema::<Root, 7>();
    assert_eq!(root.version, 7);
    let SchemaKind::Record(fields) = root.kind else {
        panic!("record schema required")
    };
    assert_eq!((fields[0].element.schema)().version, 2);
    let mut bytes = [0; 32];
    let value = Value(513);
    let len = alkahest_core::serialize_at_version::<Root, _, 7>(&value, &mut bytes).unwrap();
    assert_eq!(&bytes[..len], &[1, 2]);
    assert_eq!(
        alkahest_core::deserialize_at_version::<Root, Value, 7>(&bytes[..len]).unwrap(),
        value
    );
    let mut place = Value(0);
    alkahest_core::deserialize_in_place_at_version::<Root, _, 7>(&mut place, &bytes[..len])
        .unwrap();
    assert_eq!(place, value);
}

#[test]
#[cfg(feature = "alloc")]
fn containers_indirection_and_iterator_keep_selected_version() {
    let mut bytes = [0; 128];
    let array = [Value(257), Value(514), Value(771)];
    let value = (array, Some(Value(1028)));
    type Container = ([Word; 3], Option<Word>);
    let len = alkahest_core::serialize_at_version::<Container, _, 2>(&value, &mut bytes).unwrap();
    let decoded =
        alkahest_core::deserialize_at_version::<Container, ([Value; 3], Option<Value>), 2>(
            &bytes[..len],
        )
        .unwrap();
    assert_eq!(decoded, value);

    type Pointer = (Indirect<Word>, u8);
    let value = (Value(1285), 9u8);
    let len = alkahest_core::serialize_at_version::<Pointer, _, 2>(&value, &mut bytes).unwrap();
    let decoded =
        alkahest_core::deserialize_at_version::<Pointer, (Value, u8), 2>(&bytes[..len]).unwrap();
    assert_eq!(decoded, value);
    assert!(<Indirect<Word> as Element<2>>::SCHEMA.indirect);

    let values = alkahest_core::MakeIter(|| array.into_iter());
    let len = alkahest_core::serialize_at_version::<List<Word>, _, 2>(&values, &mut bytes).unwrap();
    let decoded =
        alkahest_core::deserialize_at_version::<List<Word>, Vec<Value>, 2>(&bytes[..len]).unwrap();
    assert_eq!(decoded, array);
    let mut place = vec![Value(0); 5];
    alkahest_core::deserialize_in_place_at_version::<List<Word>, _, 2>(&mut place, &bytes[..len])
        .unwrap();
    assert_eq!(place, array);
}

#[test]
fn lazy_and_packet_paths_keep_selected_version() {
    let mut bytes = [0; 32];
    let value = Value(513);
    let len = alkahest_core::serialize_at_version::<Word, _, 2>(&value, &mut bytes).unwrap();
    let lazy =
        alkahest_core::deserialize_at_version::<Word, Lazy<'_, Word, 2>, 2>(&bytes[..len]).unwrap();
    assert_eq!(lazy.read::<Value>().unwrap(), value);
    let mut place = Value(0);
    lazy.read_in_place(&mut place).unwrap();
    assert_eq!(place, value);

    let len =
        alkahest_core::serialize_at_version::<List<Word>, _, 2>(&[value], &mut bytes).unwrap();
    let lazy = alkahest_core::deserialize_at_version::<List<Word>, Lazy<'_, List<Word>, 2>, 2>(
        &bytes[..len],
    )
    .unwrap();
    let mut iter = lazy.iter::<Value>().unwrap();
    assert_eq!(iter.next().unwrap().unwrap(), value);
    assert!(iter.next().is_none());
    drop(iter);

    let len = alkahest_core::pack_at_version::<Word, _, 2>(&value, &mut bytes).unwrap();
    assert_eq!(len, 2);
    assert_eq!(
        alkahest_core::pack_size_at_version::<Word, _, 2>(&value),
        len
    );
    assert_eq!(
        alkahest_core::read_pack_size_at_version::<Word, 2>(&bytes).unwrap(),
        len
    );
    let (decoded, consumed) = alkahest_core::unpack_at_version::<Word, Value, 2>(&bytes).unwrap();
    assert_eq!((decoded, consumed), (value, len));
    alkahest_core::unpack_in_place_at_version::<Word, _, 2>(&mut place, &bytes).unwrap();
    assert_eq!(place, value);
}

#[test]
#[cfg(feature = "alloc")]
fn variable_packet_and_nested_indirect_children_roundtrip() {
    let mut bytes = [0; 128];
    let values = [Value(257), Value(514), Value(771)];
    let len = alkahest_core::pack_at_version::<List<Word>, _, 2>(&values, &mut bytes).unwrap();
    assert_eq!(
        alkahest_core::pack_size_at_version::<List<Word>, _, 2>(&values),
        len
    );
    let (decoded, consumed) =
        alkahest_core::unpack_at_version::<List<Word>, Vec<Value>, 2>(&bytes).unwrap();
    assert_eq!(decoded, values);
    assert_eq!(consumed, len);

    type Nested = (Indirect<List<Indirect<Word>>>, u8);
    let value = (values, 5u8);
    let len = alkahest_core::serialize_at_version::<Nested, _, 2>(&value, &mut bytes).unwrap();
    let decoded =
        alkahest_core::deserialize_at_version::<Nested, (Vec<Value>, u8), 2>(&bytes[..len])
            .unwrap();
    assert_eq!(decoded, (values.to_vec(), 5));
}

#[test]
fn default_version_entry_points_stay_usable() {
    let mut bytes = [0; 8];
    let len = alkahest_core::serialize::<u16, _>(&42u16, &mut bytes).unwrap();
    assert_eq!(
        alkahest_core::deserialize::<u16, u16>(&bytes[..len]).unwrap(),
        42
    );
    assert_eq!(schema::<u16, 0>().version, 0);
}
