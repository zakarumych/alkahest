use alkahest_core::{
    Element, Indirect, List, Never, Serialize, SizeBound, heap_size, manual_size, stack_size,
};

fn roundtrip<E, T, const SIZE_BYTES: usize>(value: &T)
where
    E: Element,
    T: Serialize<E::Formula>
        + for<'de> alkahest_core::Deserialize<'de, E::Formula>
        + PartialEq
        + core::fmt::Debug,
{
    let mut bytes = [0u8; 256];
    let len = manual_size::serialize::<E, _, SIZE_BYTES>(value, &mut bytes).expect("serialize");
    assert_eq!(manual_size::serialized_size::<E, _, SIZE_BYTES>(value), len);
    let decoded = manual_size::deserialize::<E, T, SIZE_BYTES>(&bytes[..len]).expect("deserialize");
    assert_eq!(&decoded, value);
    let len = manual_size::pack::<E, _, SIZE_BYTES>(value, &mut bytes).expect("pack");
    assert_eq!(manual_size::pack_size::<E, _, SIZE_BYTES>(value), len);
    let (decoded, consumed) = manual_size::unpack::<E, T, SIZE_BYTES>(&bytes).expect("unpack");
    assert_eq!(consumed, len);
    assert_eq!(&decoded, value);
}

#[test]
fn indirect_tuple_elements_keep_addresses_and_heap_sizes() {
    assert_eq!(stack_size::<(Indirect<u32>,), 1>(), SizeBound::Exact(1));
    assert_eq!(heap_size::<(Indirect<u32>,), 1>(), SizeBound::Exact(4));
    roundtrip::<(Indirect<u32>,), _, 1>(&(0x12345678u32,));
    roundtrip::<(Indirect<u32>, u8, Indirect<u16>), _, 8>(&(42u32, 7u8, 19u16));
}

#[test]
fn options_roundtrip_present_absent_and_zero_sized_payloads() {
    assert_eq!(stack_size::<Option<u32>, 4>(), SizeBound::Bounded(5));
    assert_eq!(heap_size::<Option<u32>, 4>(), SizeBound::Exact(0));
    assert_eq!(
        heap_size::<Option<Indirect<u32>>, 4>(),
        SizeBound::Bounded(4)
    );
    for value in [None, Some(0x12345678u32)] {
        roundtrip::<Option<u32>, _, 4>(&value);
        roundtrip::<Option<Indirect<u32>>, _, 1>(&value);
    }
    roundtrip::<Option<()>, _, 1>(&None::<()>);
    roundtrip::<Option<()>, _, 1>(&Some(()));
    roundtrip::<Option<Never>, _, 1>(&None::<Never>);
}

#[test]
fn optional_composite_elements_include_padding() {
    for first in [None, Some(17u32)] {
        for last in [None, Some(23u32)] {
            roundtrip::<(Option<u32>, u8, Option<u32>), _, 4>(&(first, 9u8, last));
            roundtrip::<(Option<Indirect<u32>>, u8, Option<Indirect<u32>>), _, 4>(&(
                first, 9u8, last,
            ));
            roundtrip::<[Option<u32>; 2], _, 4>(&[first, last]);
            roundtrip::<[Option<Indirect<u32>>; 2], _, 4>(&[first, last]);
        }
    }
}

#[test]
fn arrays_encode_lengths_only_for_variable_lists() {
    let mut bytes = [0u8; 64];
    let len = manual_size::serialize::<List<u8>, _, 1>(&[7u8, 8], &mut bytes).expect("serialize");
    assert_eq!(len, 3);
    assert_eq!(
        manual_size::deserialize::<List<u8>, Vec<u8>, 1>(&bytes[..len]).expect("list"),
        [7, 8]
    );
    let len =
        manual_size::serialize::<List<u8, 1, 3>, _, 1>(&[7u8, 8], &mut bytes).expect("serialize");
    assert_eq!(len, 3);
    assert_eq!(
        manual_size::deserialize::<List<u8, 1, 3>, Vec<u8>, 1>(&bytes[..len]).expect("list"),
        [7, 8]
    );
    let len =
        manual_size::serialize::<List<u8>, _, 1>(&[] as &[u8; 0], &mut bytes).expect("serialize");
    assert_eq!(len, 1);
    assert!(
        manual_size::deserialize::<List<u8>, Vec<u8>, 1>(&bytes[..len])
            .expect("list")
            .is_empty()
    );
    roundtrip::<[u8; 2], _, 1>(&[7u8, 8]);
    roundtrip::<[u8; 0], [u8; 0], 1>(&[]);
}

#[test]
fn tuple_allows_a_terminal_unbounded_list() {
    roundtrip::<(u32, List<u8>), _, 4>(&(17u32, vec![1u8, 2, 3]));
}

#[test]
fn optional_indirect_collections_include_padding() {
    type F = List<Option<Indirect<u32>>>;
    for values in [
        vec![],
        vec![None],
        vec![None, Some(7)],
        vec![Some(7), None, Some(11)],
    ] {
        roundtrip::<F, _, 4>(&values);
        let exact = alkahest_core::MakeIter(|| values.iter().copied());
        let unknown = alkahest_core::MakeIter(|| values.iter().copied().filter(|_| true));
        let mut bytes = [0u8; 256];
        let len = manual_size::serialize::<F, _, 4>(&exact, &mut bytes).expect("exact iterator");
        assert_eq!(
            manual_size::deserialize::<F, Vec<Option<u32>>, 4>(&bytes[..len]).expect("list"),
            values
        );
        let len =
            manual_size::serialize::<F, _, 4>(&unknown, &mut bytes).expect("unknown iterator");
        assert_eq!(
            manual_size::deserialize::<F, Vec<Option<u32>>, 4>(&bytes[..len]).expect("list"),
            values
        );
    }
}
