use alkahest_core::{
    Indirect, List, SizeBound,
    manual_size::{deserialize, deserialize_in_place, serialize},
    stack_size,
};

#[test]
fn vector_in_place_reuses_sufficient_capacity() {
    let mut place = vec![0u32; 4];
    let capacity = place.capacity();
    let pointer = place.as_ptr();
    let mut bytes = [0u8; 32];

    for values in [
        &[1u32, 2, 3, 4][..],
        &[5, 6][..],
        &[][..],
        &[7, 8, 9, 10][..],
    ] {
        let len = serialize::<List<u32>, _, 1>(&values, &mut bytes).expect("serialize list");
        deserialize_in_place::<List<u32>, _, 1>(&mut place, &bytes[..len]).expect("update list");
        assert_eq!(place, values);
        assert_eq!(place.capacity(), capacity);
        assert_eq!(place.as_ptr(), pointer);
    }
}

#[test]
fn indirect_value_supports_root_formula_and_indirect_element() {
    let mut bytes = [0u8; 16];

    let len = serialize::<(Indirect<u32>,), _, 1>(&(Indirect(0x12345678u32),), &mut bytes)
        .expect("serialize wrapped field");
    let decoded = deserialize::<(Indirect<u32>,), (Indirect<u32>,), 1>(&bytes[..len])
        .expect("deserialize wrapped field");
    assert_eq!(decoded.0.0, 0x12345678);
    let mut place = (Indirect(0u32),);
    deserialize_in_place::<(Indirect<u32>,), _, 1>(&mut place, &bytes[..len])
        .expect("update wrapped field");
    assert_eq!(place.0.0, 0x12345678);
}

#[test]
fn unbounded_tuple_field_accepts_zero_sized_trailing_field() {
    assert_eq!(stack_size::<(List<u8>, ()), 1>(), SizeBound::Unbounded);
}
