#![deny(warnings)]
#![allow(clippy::disallowed_names)]

use alkahest::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[alkahest(Formula, Serialize, SerializeRef, Deserialize)]
struct X;

#[derive(Debug)]
#[alkahest(Formula)]
struct Test<T: ?Sized> {
    a: u32,
    b: X,
    c: T,
}

#[derive(Clone, Copy, Debug)]
#[alkahest(for<U: ?Sized> SerializeRef<Test<U>> where U: Formula, for<'ser> &'ser T: Serialize<U>)]
#[alkahest(for<U: ?Sized> Serialize<Test<U>> where U: Formula, T: Serialize<U>)]
#[alkahest(for<'de, U: ?Sized> Deserialize<'de, Test<U>> where U: Formula, T: Deserialize<'de, U>)]
struct TestS<T> {
    a: u32,
    b: X,
    c: T,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[alkahest(Formula, Serialize, Deserialize)]
enum Test2 {
    Unit,
    Tuple(u32, u64),
    Struct { a: u32, b: u64 },
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[alkahest(Serialize<Test2@Unit>)]
struct Test2U;

#[derive(Clone, Copy, PartialEq, Eq)]
#[alkahest(Serialize<Test2 @Tuple>)]
struct Test2T(u32, u64);

#[derive(Clone, Copy, PartialEq, Eq)]
#[alkahest(Serialize<Test2 @Struct>)]
struct Test2S {
    a: u32,
    b: u64,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[alkahest(Serialize<Test2>)]
enum Test2E {
    Unit,
    // Tuple(u32, u64), // variants may be omitted for `Serialize`
    Struct { a: u32, b: u64 },
}

type Foo = (As<str>, As<str>);

fn main() {
    let value = ("qwe", "rty");
    let size = serialized_size::<[Foo], _>([value]);

    let mut buffer = vec![0u8; size.0];

    let size = serialize::<[Foo], _>([value], &mut buffer).unwrap();
    assert_eq!(size.0, buffer.len());

    let foo = deserialize::<[Foo], Vec<(&str, &str)>>(&buffer).unwrap();
    assert_eq!(foo, vec![("qwe", "rty")]);

    type MyFormula = Test<Vec<Vec<u32>>>;

    let value = TestS {
        a: 1,
        b: X,
        c: vec![2u8..4, 4..6],
    };

    let size = serialized_size::<MyFormula, _>(value.clone());
    let mut buffer = vec![0; size.0];
    let size = serialize::<MyFormula, _>(value.clone(), &mut buffer).unwrap();
    assert_eq!(size.0, buffer.len());
    let value = deserialize::<MyFormula, TestS<Vec<Vec<u32>>>>(&buffer).unwrap();

    assert_eq!(value.a, 1);
    assert_eq!(value.b, X);
    assert_eq!(value.c, vec![vec![2, 3], vec![4, 5]]);

    let value = Test2U;
    let size = serialized_size::<Test2, _>(value);
    let mut buffer = vec![0; size.0];
    let size = serialize::<Test2, _>(value, &mut buffer).unwrap();
    assert_eq!(size.0, buffer.len());
    let unit = deserialize::<Test2, Test2>(&buffer).unwrap();
    assert_eq!(unit, Test2::Unit);

    let value = Test2T(1, 2);
    let size = serialized_size::<Test2, _>(value);
    let mut buffer = vec![0; size.0];
    let size = serialize::<Test2, _>(value, &mut buffer).unwrap();
    assert_eq!(size.0, buffer.len());
    let structure = deserialize::<Test2, Test2>(&buffer).unwrap();
    assert_eq!(structure, Test2::Tuple(1, 2));

    let value = Test2S { a: 1, b: 2 };
    let size = serialized_size::<Test2, _>(value);
    let mut buffer = vec![0; size.0];
    let size = serialize::<Test2, _>(value, &mut buffer).unwrap();
    assert_eq!(size.0, buffer.len());
    let structure = deserialize::<Test2, Test2>(&buffer).unwrap();
    assert_eq!(structure, Test2::Struct { a: 1, b: 2 });

    let value = Test2E::Unit;
    let size = serialized_size::<Test2, _>(value);
    let mut buffer = vec![0; size.0];
    let size = serialize::<Test2, _>(value, &mut buffer).unwrap();
    assert_eq!(size.0, buffer.len());
    let unit = deserialize::<Test2, Test2>(&buffer).unwrap();
    assert_eq!(unit, Test2::Unit);

    let value = Test2E::Struct { a: 1, b: 2 };
    let size = serialized_size::<Test2, _>(value);
    let mut buffer = vec![0; size.0];
    let size = serialize::<Test2, _>(value, &mut buffer).unwrap();
    assert_eq!(size.0, buffer.len());
    let structure = deserialize::<Test2, Test2>(&buffer).unwrap();
    assert_eq!(structure, Test2::Struct { a: 1, b: 2 });
}
