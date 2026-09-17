use alkahest_core::{
    DeserializeError, Indirect, Lazy, List, Str,
    manual_size::{deserialize, deserialize_in_place, serialize},
};

#[test]
fn lazy_fixed_field_advances_cursor() {
    let mut buffer = [0; 8];
    let len = serialize::<(u32, u32), _, 1>(&(11u32, 22u32), &mut buffer).expect("serialize tuple");
    let (first, second): (Lazy<'_, u32>, u32) =
        deserialize::<(u32, u32), _, 1>(&buffer[..len]).expect("deserialize tuple");
    assert_eq!(first.read::<u32>().expect("read lazy field"), 11);
    assert_eq!(second, 22);

    let mut place = (first, 0u32);
    deserialize_in_place::<(u32, u32), _, 1>(&mut place, &buffer[..len]).expect("update tuple");
    assert_eq!(place.0.read::<u32>().expect("read updated lazy field"), 11);
    assert_eq!(place.1, 22);
}

#[test]
fn lazy_bounded_field_advances_cursor() {
    let mut buffer = [0; 9];
    let len = serialize::<(Option<u32>, u32), _, 1>(&(None::<u32>, 22u32), &mut buffer)
        .expect("serialize tuple");
    let (first, second): (Lazy<'_, Option<u32>>, u32) =
        deserialize::<(Option<u32>, u32), _, 1>(&buffer[..len]).expect("deserialize tuple");
    assert_eq!(first.read::<Option<u32>>().expect("read lazy field"), None);
    assert_eq!(second, 22);
}

#[test]
fn lazy_bounded_field_accepts_omitted_trailing_padding() {
    let mut buffer = [0; 5];
    let len =
        serialize::<Option<u32>, _, 1>(&None::<u32>, &mut buffer).expect("serialize absent value");
    assert_eq!(len, 1);
    let lazy: Lazy<'_, Option<u32>> =
        deserialize::<Option<u32>, _, 1>(&buffer[..len]).expect("capture absent value");
    assert_eq!(lazy.read::<Option<u32>>().expect("read absent value"), None);
}

#[test]
fn lazy_exact_field_rejects_truncated_input() {
    assert!(deserialize::<u32, Lazy<'_, u32>, 1>(&[0; 3]).is_err());
}

#[test]
fn trivial_values_reject_truncated_input_without_mutating_place() {
    let input = 17u32.to_le_bytes();
    for len in 0..input.len() {
        assert!(matches!(
            deserialize::<u32, u32, 1>(&input[..len]),
            Err(DeserializeError::WrongLength)
        ));
        let mut place = 42u32;
        assert!(matches!(
            deserialize_in_place::<u32, _, 1>(&mut place, &input[..len]),
            Err(DeserializeError::WrongLength)
        ));
        assert_eq!(place, 42);
    }
}

#[test]
fn lazy_unbounded_string_preserves_borrowed_input() {
    let source = String::from("lazy string");
    let mut buffer = [0; 32];
    let len = serialize::<Str, _, 1>(&source, &mut buffer).expect("serialize string");
    let lazy: Lazy<'_, Str> = deserialize::<Str, _, 1>(&buffer[..len]).expect("capture string");
    let value = lazy.read::<&str>().expect("read string");
    assert_eq!(value, source);
    assert!(value.as_ptr() >= buffer.as_ptr());
    assert!(value.as_ptr() < buffer[len..].as_ptr());
}

#[test]
fn lazy_unbounded_string_defers_payload_validation() {
    let lazy: Lazy<'_, Str> = deserialize::<Str, _, 1>(&[0xff, 1]).expect("capture string");
    assert!(matches!(
        lazy.read::<&str>(),
        Err(DeserializeError::NonUtf8(_))
    ));
}

#[test]
fn lazy_compound_field_preserves_heap_addresses() {
    type Inner = (Indirect<u32>, u32);
    type Outer = (Inner, u32);
    let mut buffer = [0; 32];
    let len = serialize::<Outer, _, 1>(&((7u32, 8u32), 9u32), &mut buffer)
        .expect("serialize indirect field");
    let (lazy, last): (Lazy<'_, Inner>, u32) =
        deserialize::<Outer, _, 1>(&buffer[..len]).expect("capture compound field");
    assert_eq!(last, 9);
    assert_eq!(
        lazy.read::<(u32, u32)>().expect("read compound field"),
        (7, 8)
    );
}

#[test]
fn vector_in_place_shrinks_empties_and_grows() {
    let mut place = vec![1u32, 2, 3];
    for source in [vec![7u32], vec![], vec![8, 9, 10, 11]] {
        let mut buffer = [0; 32];
        let len = serialize::<List<u32>, _, 1>(&source, &mut buffer).expect("serialize list");
        deserialize_in_place::<List<u32>, _, 1>(&mut place, &buffer[..len]).expect("update vector");
        assert_eq!(place, source);
    }
}

#[test]
fn hostile_vector_lengths_are_rejected_before_allocation() {
    for len in [usize::MAX, usize::MAX / 8, 1] {
        let input = (len as u64).to_le_bytes();
        assert!(matches!(
            deserialize::<List<u64>, Vec<u64>, 8>(&input),
            Err(DeserializeError::WrongLength)
        ));
        let mut place = vec![42u64];
        let capacity = place.capacity();
        assert!(matches!(
            deserialize_in_place::<List<u64>, _, 8>(&mut place, &input),
            Err(DeserializeError::WrongLength)
        ));
        assert_eq!(place, [42]);
        assert_eq!(place.capacity(), capacity);
    }
}

#[test]
fn bounded_vector_accepts_omitted_final_padding() {
    let source = vec![Some(7u32), None];
    let mut buffer = [0; 16];
    let len = serialize::<List<Option<u32>>, _, 1>(&source, &mut buffer).expect("serialize list");
    let result: Vec<Option<u32>> =
        deserialize::<List<Option<u32>>, _, 1>(&buffer[..len]).expect("deserialize list");
    assert_eq!(result, source);
}

#[test]
fn hostile_bounded_vector_length_is_rejected() {
    let input = u64::MAX.to_le_bytes();
    assert!(deserialize::<List<Option<u32>>, Vec<Option<u32>>, 8>(&input).is_err());
    let mut place = vec![Some(42u32)];
    assert!(deserialize_in_place::<List<Option<u32>>, _, 8>(&mut place, &input).is_err());
    assert_eq!(place, [Some(42)]);
}
