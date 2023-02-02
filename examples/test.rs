use alkahest::{
    deserialize, serialize, serialized_size, Deserialize, Schema, Serialize, SizedSchema,
};

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, SizedSchema, Serialize, Deserialize,
)]
struct X;

#[derive(Schema)]
struct Test<T: ?Sized> {
    a: u32,
    b: X,
    c: T,
}

#[derive(Debug, Serialize, Deserialize)]
#[alkahest(serialize(for<'ser, U> Test<[U]> where U: SizedSchema + 'ser, T: 'ser, &'ser T: Serialize<U>))]
#[alkahest(serialize(noref(for<U> Test<[U]> where U: SizedSchema, T: Serialize<U>)))]
#[alkahest(deserialize(for<'de, U> Test<[U]> where U: SizedSchema, T: Deserialize<'de, U>))]
struct TestS<T> {
    a: u32,
    b: X,
    c: Vec<T>,
}

fn main() {
    let value = TestS {
        a: 1,
        b: X,
        c: vec![2, 3],
    };

    let size = serialized_size::<Test<[u32]>, _>(&value);
    println!("size: {}", size);

    let mut buffer = vec![0; size];

    let size = serialize::<Test<[u32]>, _>(&value, &mut buffer).unwrap();
    assert_eq!(size, buffer.len());

    let (value, size) = deserialize::<Test<[u32]>, TestS<u32>>(&buffer).unwrap();
    assert_eq!(size, buffer.len());

    assert_eq!(value.a, 1);
    assert_eq!(value.b, X);
    assert_eq!(value.c, vec![2, 3]);

    println!("{:?}", value);
}
