use crate::{
    buffer::BufferExhausted,
    bytes::Bytes,
    deserialize::{deserialize, deserialize_in_place, value_size, Deserialize},
    formula::Formula,
    lazy::Lazy,
    r#as::As,
    reference::Ref,
    serialize::{header_size, serialize, serialize_or_size, serialized_size, Serialize},
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

    let header_size = header_size::<F>();
    assert!(header_size <= size);

    match (F::HEAPLESS, F::EXACT_SIZE, F::MAX_STACK_SIZE) {
        (true, true, Some(max_stack)) => assert_eq!(header_size + max_stack, size),
        (true, false, Some(max_stack)) => assert!(header_size + max_stack >= size),
        _ => {}
    }

    match serialize_or_size::<F, _>(value, &mut []) {
        Ok(_) => assert_eq!(size, 0),
        Err(err) => assert_eq!(err.required, size),
    }

    if size > 0 {
        match serialize_or_size::<F, _>(value, &mut buffer[..size - 1]) {
            Ok(_) => panic!("expected error"),
            Err(err) => assert_eq!(err.required, size),
        }
    }

    let size1 = serialize_or_size::<F, _>(value, buffer).expect("expected success");
    assert_eq!(size, size1);
    assert_eq!(size, value_size::<F>(&buffer).expect("expected success"));
    let buffer2 = &mut buffer[size..];

    match serialize::<F, _>(value, &mut []) {
        Ok(_) => assert_eq!(size, 0),
        Err(BufferExhausted) => {}
    }

    if size > 0 {
        match serialize::<F, _>(value, &mut buffer2[..size - 1]) {
            Ok(_) => panic!("expected error"),
            Err(BufferExhausted) => {}
        }
    }

    let size2 = serialize::<F, _>(value, buffer2).expect("expected success");
    assert_eq!(size, size2);
    assert_eq!(size, value_size::<F>(&buffer2).expect("expected success"));

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
            test_type::<[$t], [$t], Lazy<[$t]>>(&[$v; 3], &mut $buffer, |x, y| {
                y.iter::<$t>().zip(x.iter()).all(|(x, y)| x.unwrap() == *y)
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
    // test_type::<Ref<()>, (), ()>(&(), &mut buffer, |x, y| x == y);
    // test_type::<Ref<u32>, u32, u32>(&1, &mut buffer, |x, y| x == y);
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

#[cfg(all(feature = "alloc", feature = "derive"))]
#[test]
fn test_enums() {
    use crate::{Deserialize, Formula, Serialize};
    use alloc::vec::Vec;

    #[derive(Formula)]
    enum TestFormula {
        Foo { a: Ref<u32> },
        Bar { c: Vec<u32>, d: Vec<Vec<u32>> },
    }

    #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
    #[alkahest(TestFormula)]
    enum TestData {
        Foo { a: u32 },
        Bar { c: Vec<u32>, d: Vec<Vec<u32>> },
    }

    #[derive(Deserialize)]
    #[alkahest(TestFormula)]
    enum TestDataLazy<'a> {
        Foo {
            a: u32,
        },
        Bar {
            c: Lazy<'a, [u32]>,
            d: Lazy<'a, [Vec<u32>]>,
        },
    }

    let data = TestData::Foo { a: 1 };
    let mut bytes = [0u8; 1024];

    alkahest::serialize::<TestFormula, _>(data, &mut bytes).unwrap();
    let (data, _) = alkahest::deserialize::<TestFormula, TestData>(&bytes).unwrap();
    assert_eq!(data, TestData::Foo { a: 1 });
}

#[cfg(all(feature = "alloc", feature = "derive"))]
#[test]
fn test_bench() {
    use crate::{Deserialize, Formula, Serialize};
    use alloc::vec::Vec;

    #[derive(Formula)]
    enum TestFormula {
        Foo { a: Ref<u32>, b: Ref<u32> },
        Bar { c: Vec<u32>, d: Vec<Vec<u32>> },
    }

    #[derive(Serialize, Deserialize)]
    #[alkahest(TestFormula)]
    enum TestData {
        Foo { a: u32, b: u32 },
        Bar { c: Vec<u32>, d: Vec<Vec<u32>> },
    }

    #[derive(Deserialize)]
    #[alkahest(TestFormula)]
    enum TestDataLazy<'a> {
        Foo {
            a: u32,
            b: u32,
        },
        Bar {
            c: Lazy<'a, [u32]>,
            d: Lazy<'a, [Vec<u32>]>,
        },
    }

    let data = TestData::Foo { a: 1, b: 2 };
    let mut bytes = [0u8; 1024];

    alkahest::serialize::<TestFormula, _>(data, &mut bytes).unwrap();
    let (_data, _) = alkahest::deserialize::<TestFormula, TestDataLazy>(&bytes).unwrap();
}

#[cfg(feature = "alloc")]
#[test]
fn test_slice_of_slice() {
    use alloc::vec::Vec;

    let mut buffer = [0u8; 256];
    test_type::<[As<[u8]>], [&[u8]], Vec<Vec<u8>>>(
        &[&[1, 2, 3], &[5, 6, 7, 8]],
        &mut buffer,
        |x, y| x.iter().zip(y.iter()).all(|(x, y)| x == y),
    );
}

#[test]
fn test_size() {
    const REFS: usize = 4;
    const REF_SIZE: usize = cfg!(feature = "fixed8") as usize
        + cfg!(feature = "fixed16") as usize * 2
        + cfg!(feature = "fixed32") as usize * 4
        + cfg!(feature = "fixed64") as usize * 8;

    const PAYLOAD: usize = 6;
    const SIZE: usize = REFS * REF_SIZE + PAYLOAD;

    let mut buffer = [0u8; SIZE];

    serialize::<[As<str>], _>(["qwe", "rty"], &mut buffer).unwrap();
}

#[cfg(all(feature = "derive", feature = "alloc"))]
#[test]
fn test_packet() {
    use alkahest_proc::{Deserialize, Formula, Serialize};
    use alloc::{string::String, vec, vec::Vec};

    #[derive(Debug, Clone, Formula, Serialize, Deserialize)]
    pub enum GameMessage {
        Client(ClientMessage),
        Server(ServerMessage),
    }

    #[derive(Debug, Clone, Formula, Serialize, Deserialize)]
    pub enum ClientMessage {
        ClientData { nickname: String, clan: String },
        Chat(String),
    }

    #[derive(Debug, Clone, Formula, Serialize, Deserialize)]
    pub enum ServerMessage {
        ServerData,
        ClientChat { client_id: u64, message: String },
    }

    #[derive(Debug, Formula, Serialize, Deserialize)]
    pub struct NetPacket<G> {
        pub game_messages: Vec<G>,
    }

    let mut buffer = [0u8; 1024];
    alkahest::serialize::<NetPacket<GameMessage>, _>(
        NetPacket {
            game_messages: vec![
                // GameMessage::Client(ClientMessage::ClientData {
                //     nickname: "qwe".into(),
                //     clan: "rty".into(),
                // }),
                // GameMessage::Client(ClientMessage::Chat("zxc".into())),
                // GameMessage::Server(ServerMessage::ClientChat {
                //     client_id: 1,
                //     message: "asd".into(),
                // }),
                GameMessage::Server(ServerMessage::ServerData),
            ],
        },
        &mut buffer,
    )
    .unwrap();
}
