#[cfg(all(feature = "alloc", feature = "derive"))]
mod net;

#[cfg(feature = "alloc")]
use alloc::{collections::VecDeque, vec, vec::Vec};

#[cfg(all(feature = "derive", feature = "alloc"))]
use alkahest_proc::{Deserialize, Formula, Serialize};

use crate::{
    buffer::BufferExhausted,
    bytes::Bytes,
    deserialize::{deserialize, deserialize_in_place, value_size, Deserialize},
    formula::{reference_size, Formula},
    lazy::Lazy,
    r#as::As,
    reference::Ref,
    serialize::{serialize, serialize_or_size, serialized_size, Serialize},
    vlq::Vlq,
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

    let header_size = reference_size::<F>();
    assert!(header_size <= size);

    match (F::HEAPLESS, F::EXACT_SIZE, F::MAX_STACK_SIZE) {
        (true, true, Some(max_stack)) => assert_eq!(header_size + max_stack, size),
        (true, false, Some(max_stack)) => assert!(header_size + max_stack >= size),
        _ => {}
    }

    // match serialize_or_size::<F, _>(value, &mut []) {
    //     Ok(_) => assert_eq!(size, 0),
    //     Err(err) => assert_eq!(err.required, size),
    // }

    // if size > 0 {
    //     match serialize_or_size::<F, _>(value, &mut buffer[..size - 1]) {
    //         Ok(_) => panic!("expected error"),
    //         Err(err) => assert_eq!(err.required, size),
    //     }
    // }

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
    test_type::<[As<[u8]>], [&[u8]], Vec<Vec<u8>>>(&[], &mut buffer, |x, y| {
        x.iter().zip(y.iter()).all(|(x, y)| x == y)
    });

    test_type::<[As<[u8]>], [&[u8]], Vec<Vec<u8>>>(&[&[], &[], &[]], &mut buffer, |x, y| {
        x.iter().zip(y.iter()).all(|(x, y)| x == y)
    });

    test_type::<[As<[u8]>], [&[u8]], Vec<Vec<u8>>>(
        &[&[1, 2, 3], &[5, 6, 7, 8]],
        &mut buffer,
        |x, y| x.iter().zip(y.iter()).all(|(x, y)| x == y),
    );

    test_type::<[As<[u8]>], [&[u8]], Vec<Vec<u8>>>(&[&[1, 2], &[], &[3]], &mut buffer, |x, y| {
        x.iter().zip(y.iter()).all(|(x, y)| x == y)
    });
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
    use alloc::{string::String, vec, vec::Vec};

    #[derive(Debug, Clone, Formula, Serialize, Deserialize)]
    pub enum GameMessage {
        Client(ClientMessage),
        Server(ServerMessage),
    }

    assert_eq!(
        <GameMessage as Formula>::EXACT_SIZE,
        false,
        "Enum with non-EXACT_SIZE variants are not EXACT_SIZE"
    );

    #[derive(Debug, Clone, Formula, Serialize, Deserialize)]
    pub enum ClientMessage {
        ClientData { nickname: String, clan: String },
        Chat(String),
    }

    assert_eq!(
        <ClientMessage as Formula>::EXACT_SIZE,
        false,
        "Enums with differently sized variants are not EXACT_SIZE"
    );

    #[derive(Debug, Clone, Formula, Serialize, Deserialize)]
    pub enum ServerMessage {
        ServerData,
        ClientChat { client_id: u64, message: String },
    }

    assert_eq!(
        <ServerMessage as Formula>::EXACT_SIZE,
        false,
        "Enums with differently sized variants are not EXACT_SIZE"
    );

    #[derive(Debug, Formula, Serialize, Deserialize)]
    pub struct NetPacket<G> {
        pub game_messages: Vec<G>,
    }

    let mut buffer = [0u8; 1024];
    alkahest::serialize::<NetPacket<GameMessage>, _>(
        NetPacket {
            game_messages: vec![
                GameMessage::Client(ClientMessage::ClientData {
                    nickname: "qwe".into(),
                    clan: "rty".into(),
                }),
                GameMessage::Client(ClientMessage::Chat("zxc".into())),
                GameMessage::Server(ServerMessage::ClientChat {
                    client_id: 1,
                    message: "asd".into(),
                }),
                GameMessage::Server(ServerMessage::ServerData),
            ],
        },
        &mut buffer,
    )
    .unwrap();
}

#[cfg(feature = "alloc")]
#[test]
fn test_zst_slice() {
    use alloc::{vec, vec::Vec};

    let mut buffer = [0u8; 256];
    test_type::<[()], [()], Vec<()>>(&[(), (), ()], &mut buffer, |x, y| x == y);
    test_type::<[()], Vec<()>, Vec<()>>(&vec![()], &mut buffer, |x, y| x == y);
}

#[cfg(all(feature = "alloc", feature = "derive"))]
#[test]
fn test_ref_in_enum() {
    use alloc::{
        string::{String, ToString},
        vec::Vec,
    };

    use alkahest_proc::{Deserialize, Formula, Serialize};

    #[derive(Debug, PartialEq, Eq, Formula, Serialize, Deserialize)]
    enum Test {
        B([u64; 16]),
        A(String),
    }

    let value = Test::A("qwerty".to_string());

    let mut buffer = [0u8; 256];
    let size = serialize::<[Test], _>([&value], &mut buffer).unwrap();
    let (data, _) = deserialize::<[Test], Vec<Test>>(&buffer[..size]).unwrap();

    assert_eq!(data, [value]);
}

#[test]
fn test_vlq() {
    let mut buffer = [0u8; 1024];

    let u8s = [0u8, 1, 2, 3, 15, 16, 127, 128, 255];

    let u32s = [
        0u32, 1, 2, 3, 15, 16, 127, 128, 255, 256, 511, 512, 541, 1235, 145436, 1415156,
    ];

    let u128s = [
        0u128,
        1,
        2,
        3,
        15,
        16,
        127,
        128,
        255,
        256,
        511,
        512,
        541,
        1235,
        145436,
        1415156,
        8686126246,
        451395861346,
        8513556350934828745,
        35815984654386789363134,
        784467440737095516151415,
        335135563509348287454252346983435251968,
    ];

    for i in u8s {
        let size = serialize::<Vlq, _>(i, &mut buffer).unwrap();
        let (de, _) = deserialize::<Vlq, u32>(&buffer[..size]).unwrap();
        assert_eq!(de, u32::from(i));
    }
    for i in u32s {
        let size = serialize::<Vlq, _>(i, &mut buffer).unwrap();
        let (de, _) = deserialize::<Vlq, u64>(&buffer[..size]).unwrap();
        assert_eq!(de, u64::from(i));
    }
    for i in u128s {
        let size = serialize::<Vlq, _>(i, &mut buffer).unwrap();
        let (de, _) = deserialize::<Vlq, u128>(&buffer[..size]).unwrap();
        assert_eq!(de, i);
    }
}

#[cfg(feature = "bincoded")]
#[test]
fn test_bincoded() {
    use serde::{de::*, ser::*};

    use crate::bincoded::*;

    struct Value(u32);

    impl Serialize for Value {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            <u32 as Serialize>::serialize(&self.0, serializer)
        }
    }

    impl<'de> Deserialize<'de> for Value {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            <u32 as Deserialize<'de>>::deserialize(deserializer).map(Value)
        }
    }

    let mut buffer = [0u8; 1024];

    let size = serialize::<Bincode, _>(Value(102414), &mut buffer).unwrap();
    let (de, _) = deserialize::<Bincode, Value>(&buffer[..size]).unwrap();
    assert_eq!(de.0, 102414);
}

#[test]
fn test_zero_sized_arrays() {
    serialize::<[u8; 0], [u8; 0]>([], &mut []).unwrap();
    serialize::<[(); 1], [(); 1]>([()], &mut []).unwrap();

    let ([], _) = deserialize::<[u8; 0], [u8; 0]>(&[]).unwrap();
    let ([()], _) = deserialize::<[(); 1], [(); 1]>(&[]).unwrap();

    #[cfg(feature = "alloc")]
    {
        deserialize::<[u8; 0], Vec<u8>>(&[]).unwrap();
        deserialize::<[u8; 0], VecDeque<u8>>(&[]).unwrap();
    }
}
