use crate::{
    bytes::Bytes,
    deserialize::{deserialize, deserialize_in_place, value_size, Deserialize},
    formula::Formula,
    lazy::LazySlice,
    r#as::As,
    reference::Ref,
    serialize::{
        serialize, serialize_or_size, serialized_size, BufferExhausted, Serialize, HEADER_SIZE,
    },
};

fn test_type<'a, F, T, D>(value: &T, buffer: &'a mut [u8], eq: impl Fn(&T, &D) -> bool)
where
    F: Formula + ?Sized,
    T: ?Sized,
    for<'x> &'x T: Serialize<F>,
    D: Deserialize<'a, F>,
{
    let size = serialized_size::<F, _>(value);

    if size * 2 > buffer.len() {
        panic!("Test data is too large");
    }

    assert!(HEADER_SIZE <= size);

    match (F::HEAPLESS, F::EXACT_SIZE, F::MAX_STACK_SIZE) {
        (true, true, Some(max_stack)) => assert_eq!(HEADER_SIZE + max_stack, size),
        (true, false, Some(max_stack)) => assert!(HEADER_SIZE + max_stack >= size),
        _ => {}
    }

    match serialize_or_size::<F, _>(value, &mut []) {
        Ok(_) => panic!("expected error"),
        Err(err) => assert_eq!(err.required, size),
    }

    match serialize_or_size::<F, _>(value, &mut buffer[..size - 1]) {
        Ok(_) => panic!("expected error"),
        Err(err) => assert_eq!(err.required, size),
    }

    let size1 = serialize_or_size::<F, _>(value, buffer).expect("expected success");
    assert_eq!(size, size1);
    assert_eq!(size, value_size(&buffer).expect("expected success"));
    let buffer2 = &mut buffer[size..];

    match serialize::<F, _>(value, &mut []) {
        Ok(_) => panic!("expected error"),
        Err(BufferExhausted) => {}
    }

    match serialize::<F, _>(value, &mut buffer2[..size - 1]) {
        Ok(_) => panic!("expected error"),
        Err(BufferExhausted) => {}
    }

    let size2 = serialize::<F, _>(value, buffer2).expect("expected success");
    assert_eq!(size, size2);
    assert_eq!(size, value_size(&buffer2).expect("expected success"));

    let buffer = &buffer[..];
    let buffer2 = &buffer[size..];

    let (mut deval, desize) = deserialize::<F, D>(buffer).expect("expected success");
    assert_eq!(size, desize);
    assert!(eq(value, &deval));

    let desize = deserialize_in_place::<F, _>(&mut deval, buffer).expect("expected success");
    assert_eq!(size, desize);
    assert!(eq(value, &deval));

    let (mut deval, desize) = deserialize::<F, D>(buffer2).expect("expected success");
    assert_eq!(size, desize);
    assert!(eq(value, &deval));

    let desize = deserialize_in_place::<F, _>(&mut deval, buffer2).expect("expected success");
    assert_eq!(size, desize);
    assert!(eq(value, &deval));
}

#[test]
fn test_primitives() {
    macro_rules! test_primitive {
        ($buffer:expr, $t:ty = $v:expr) => {
            test_type::<$t, $t, $t>(&$v, &mut $buffer, |x, y| *x == *y);
        };
    }

    let mut buffer = [0u8; 48];

    test_primitive!(buffer, u8 = 0);
    test_primitive!(buffer, u16 = 0);
    test_primitive!(buffer, u32 = 0);
    test_primitive!(buffer, u64 = 0);
    test_primitive!(buffer, u128 = 0);
    test_primitive!(buffer, i8 = 0);
    test_primitive!(buffer, i16 = 0);
    test_primitive!(buffer, i32 = 0);
    test_primitive!(buffer, i64 = 0);
    test_primitive!(buffer, i128 = 0);
}

#[test]
fn test_array() {
    macro_rules! test_primitive {
        ($buffer:expr, $t:ty = $v:expr) => {
            test_type::<[$t; 3], [$t; 3], [$t; 3]>(&[$v; 3], &mut $buffer, |x, y| *x == *y);
        };
    }

    let mut buffer = [0u8; 256];

    test_primitive!(buffer, u8 = 0);
    test_primitive!(buffer, u16 = 0);
    test_primitive!(buffer, u32 = 0);
    test_primitive!(buffer, u64 = 0);
    test_primitive!(buffer, u128 = 0);
    test_primitive!(buffer, i8 = 0);
    test_primitive!(buffer, i16 = 0);
    test_primitive!(buffer, i32 = 0);
    test_primitive!(buffer, i64 = 0);
    test_primitive!(buffer, i128 = 0);
}

#[test]
fn test_slice() {
    macro_rules! test_primitive {
        ($buffer:expr, $t:ty = $v:expr) => {
            test_type::<[$t], [$t], LazySlice<$t>>(&[$v; 3], &mut $buffer, |x, y| {
                y.iter().zip(x.iter()).all(|(x, y)| x.unwrap() == *y)
            });
        };
    }

    let mut buffer = [0u8; 256];

    test_primitive!(buffer, u8 = 0);
    test_primitive!(buffer, u16 = 0);
    test_primitive!(buffer, u32 = 0);
    test_primitive!(buffer, u64 = 0);
    test_primitive!(buffer, u128 = 0);
    test_primitive!(buffer, i8 = 0);
    test_primitive!(buffer, i16 = 0);
    test_primitive!(buffer, i32 = 0);
    test_primitive!(buffer, i64 = 0);
    test_primitive!(buffer, i128 = 0);
}

#[test]
fn test_ref() {
    let mut buffer = [0u8; 256];
    test_type::<Ref<()>, (), ()>(&(), &mut buffer, |x, y| x == y);
    test_type::<Ref<u32>, u32, u32>(&1, &mut buffer, |x, y| x == y);
    test_type::<Ref<str>, str, &str>("qwe", &mut buffer, |x, y| x == *y);
}

#[test]
fn test_complex_tuple() {
    type Formula = (u8, (u16, Bytes), As<str>, Ref<(u32, As<str>, str)>);
    type Serialize<'ser> = (
        u8,
        (u16, &'ser [u8]),
        &'ser str,
        (u32, &'ser str, &'ser str),
    );
    type Deserialize<'de> = (u8, (u16, &'de [u8]), &'de str, (u32, &'de str, &'de str));

    let mut buffer = [0u8; 256];
    test_type::<Formula, Serialize, Deserialize>(
        &(1, (2, &[1, 2, 3, 4]), "qwe", (11, "rty", "asd")),
        &mut buffer,
        |x, y| x == y,
    );
}

#[cfg(feature = "alloc")]
#[test]
fn test_vec() {
    use alloc::{vec, vec::Vec};

    let mut buffer = [0u8; 256];
    test_type::<Vec<u8>, Vec<u8>, Vec<u8>>(&vec![1, 2, 3, 4], &mut buffer, |x, y| x == y);
}
