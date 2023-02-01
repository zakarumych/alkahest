use alkahest::{
    deserialize, serialize, serialized_size, Deserialize, Schema, Serialize, SizedSchema,
};

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, SizedSchema, Serialize, Deserialize,
)]
struct X;

#[derive(Clone, Debug, Schema, Serialize, Deserialize)]
struct Test {
    a: u32,
    b: X,
    c: Vec<u32>,
}

fn main() {
    let value = Test {
        a: 1,
        b: X,
        c: vec![2, 3],
    };

    let size = serialized_size::<Test, _>(&value);
    println!("size: {}", size);

    let mut buffer = vec![0; size];

    let size = serialize::<Test, _>(&value, &mut buffer).unwrap();
    assert_eq!(size, buffer.len());

    let (value, size) = deserialize::<Test, Test>(&buffer).unwrap();
    assert_eq!(size, buffer.len());

    assert_eq!(value.a, 1);
    // assert_eq!(value.b, X);
    assert_eq!(value.c, vec![2, 3]);

    println!("{:?}", value);
}
