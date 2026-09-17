use alkahest_core::{
    Array, Formula, Indirect, List, Never, Serialize, SizeBound, heap_size, manual_size, stack_size,
};

fn roundtrip<F, T, const SIZE_BYTES: usize>(value: &T)
where
    F: Formula,
    T: Serialize<F> + for<'de> alkahest_core::Deserialize<'de, F> + PartialEq + core::fmt::Debug,
{
    let mut bytes = [0u8; 256];
    let len = manual_size::serialize::<F, _, SIZE_BYTES>(value, &mut bytes).expect("serialize");
    assert_eq!(manual_size::serialized_size::<F, _, SIZE_BYTES>(value), len);
    let decoded = manual_size::deserialize::<F, T, SIZE_BYTES>(&bytes[..len]).expect("deserialize");
    assert_eq!(&decoded, value);
    let len = manual_size::pack::<F, _, SIZE_BYTES>(value, &mut bytes).expect("pack");
    assert_eq!(manual_size::pack_size::<F, _, SIZE_BYTES>(value), len);
    let (decoded, consumed) = manual_size::unpack::<F, T, SIZE_BYTES>(&bytes).expect("unpack");
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
fn zero_sized_indirect_targets_use_no_pointer_slots() {
    assert_eq!(stack_size::<(Indirect<()>,), 1>(), SizeBound::Exact(0));
    assert_eq!(heap_size::<(Indirect<()>,), 1>(), SizeBound::Exact(0));
    roundtrip::<(Indirect<()>,), _, 1>(&((),));
    roundtrip::<(Indirect<()>,), _, 16>(&((),));
    for value in [None, Some(17u32)] {
        roundtrip::<(Option<Indirect<u32>>, Indirect<()>), _, 1>(&(value, ()));
    }
    roundtrip::<(Indirect<u32>, Indirect<()>, u8), _, 1>(&(0x01020304u32, (), 7u8));
    roundtrip::<(Indirect<()>, Indirect<u32>), _, 1>(&((), 0x01020304u32));
    let mut bytes = [0xa5; 6];
    let len = manual_size::serialize::<(Indirect<u32>, Indirect<()>, u8), _, 1>(
        &(0x01020304u32, (), 7u8),
        &mut bytes,
    )
    .expect("serialize mixed pointers");
    assert_eq!(len, 6);
    assert_eq!(bytes, [4, 3, 2, 1, 7, 4]);
}

struct EmptyBound<const MAX: usize>;

impl<const MAX: usize> Formula for EmptyBound<MAX> {
    type StackSize<const SIZE_BYTES: usize> = alkahest_core::BoundedSize<MAX>;
    type HeapSize<const SIZE_BYTES: usize> = alkahest_core::BoundedSize<0>;
    const INHABITED: bool = true;
}

impl<const MAX: usize> Serialize<EmptyBound<MAX>> for () {
    fn serialize<S: alkahest_core::Serializer>(&self, _: S) -> Result<(), S::Error> {
        assert_ne!(MAX, 0, "statically empty targets must not be serialized");
        Ok(())
    }

    fn size_hint<const SIZE_BYTES: usize>(&self) -> Option<alkahest_core::Sizes> {
        assert_ne!(MAX, 0, "statically empty targets must not need a size hint");
        None
    }
}

impl<'de, const MAX: usize> alkahest_core::Deserialize<'de, EmptyBound<MAX>> for () {
    fn deserialize<D: alkahest_core::Deserializer<'de>>(
        _: D,
    ) -> Result<Self, alkahest_core::DeserializeError> {
        Ok(())
    }

    fn deserialize_in_place<D: alkahest_core::Deserializer<'de>>(
        &mut self,
        _: D,
    ) -> Result<(), alkahest_core::DeserializeError> {
        Ok(())
    }
}

fn check_bounded_empty<const SIZE_BYTES: usize>() {
    use alkahest_core::{Element, Lazy, Sizes};
    type F = (Indirect<EmptyBound<0>>, u8);
    assert_eq!(
        stack_size::<Indirect<EmptyBound<0>>, SIZE_BYTES>(),
        SizeBound::Exact(0)
    );
    assert_eq!(
        heap_size::<Indirect<EmptyBound<0>>, SIZE_BYTES>(),
        SizeBound::Exact(0)
    );
    assert_eq!(
        <Indirect<EmptyBound<0>> as Element>::size_hint::<_, SIZE_BYTES>(&()),
        Some(Sizes::ZERO)
    );
    let mut bytes = [0xa5; 1];
    assert_eq!(
        manual_size::serialize::<F, _, SIZE_BYTES>(&((), 7u8), &mut bytes)
            .expect("serialize bounded zero"),
        1
    );
    assert_eq!(bytes, [7]);
    let mut value =
        manual_size::deserialize::<F, ((), u8), SIZE_BYTES>(&bytes).expect("decode bounded zero");
    manual_size::deserialize_in_place::<F, _, SIZE_BYTES>(&mut value, &bytes)
        .expect("update bounded zero");
    assert_eq!(value, ((), 7));
    let (lazy, value): (Lazy<'_, EmptyBound<0>>, u8) =
        manual_size::deserialize::<F, _, SIZE_BYTES>(&bytes).expect("lazy bounded zero");
    assert_eq!(value, 7);
    lazy.read::<()>().expect("read bounded zero");
    lazy.read_in_place(&mut ()).expect("update bounded zero");
}

#[test]
fn statically_empty_indirection_needs_neither_hint_nor_bytes() {
    check_bounded_empty::<1>();
    check_bounded_empty::<16>();
}

#[test]
fn runtime_empty_indirection_retains_address() {
    type F = (Indirect<EmptyBound<1>>,);
    assert_eq!(stack_size::<F, 1>(), SizeBound::Exact(1));
    let mut bytes = [0xa5; 1];
    assert_eq!(
        manual_size::serialize::<F, _, 1>(&((),), &mut bytes).expect("serialize runtime empty"),
        1
    );
    assert_eq!(bytes, [0]);
    manual_size::deserialize::<F, ((),), 1>(&bytes).expect("decode runtime empty");
    assert!(manual_size::deserialize::<F, ((),), 1>(&[]).is_err());
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
            roundtrip::<Array<Option<u32>, 2>, _, 4>(&[first, last]);
            roundtrip::<Array<Option<Indirect<u32>>, 2>, _, 4>(&[first, last]);
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
    roundtrip::<Array<u8, 2>, _, 1>(&[7u8, 8]);
    roundtrip::<Array<u8, 0>, [u8; 0], 1>(&[]);
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
